terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 7.3"
    }
  }
  backend "gcs" {
    bucket = ""
    prefix = ""
  }
}

variable "project_id" {}
variable "region" {}

variable "network_name" {}

variable "db_instance_name" {}
variable "db_name" {}
variable "db_user" {
  type = object({
    username           = string
    password_secret_id = string
  })
}

variable "backend" {
  type = object({
    service_name = string
    image_url    = string
  })
}

variable "frontend" {
  type = object({
    service_name = string
    image_url    = string
  })
}

variable "admin" {
  type = object({
    username           = string
    password_secret_id = string
  })
}
variable "session_key_secret_id" {}

variable "github" {
  type = object({
    username = string
    repo     = string
  })
}

provider "google" {
  project = var.project_id
  region  = var.region
}

data "google_compute_network" "default" {
  name = var.network_name
}

resource "google_compute_global_address" "database" {
  name          = "chronicle-db"
  purpose       = "VPC_PEERING"
  address_type  = "INTERNAL"
  prefix_length = 24
  network       = data.google_compute_network.default.id
}

resource "google_service_networking_connection" "database" {
  network                 = data.google_compute_network.default.id
  service                 = "servicenetworking.googleapis.com"
  reserved_peering_ranges = [google_compute_global_address.database.name]
}

resource "google_sql_database_instance" "default" {
  name                = var.db_instance_name
  region              = var.region
  database_version    = "POSTGRES_17"
  deletion_protection = true
  settings {
    edition = "ENTERPRISE"
    tier    = "db-f1-micro"
    ip_configuration {
      ipv4_enabled    = false
      private_network = data.google_compute_network.default.self_link
      #   enable_private_path_for_google_cloud_services = true
    }
  }
  depends_on = [google_service_networking_connection.database]
}

resource "google_sql_database" "default" {
  name     = var.db_name
  instance = google_sql_database_instance.default.name
}

data "google_secret_manager_secret_version" "db_password" {
  secret  = var.db_user.password_secret_id
  version = "latest"
}

resource "google_sql_user" "default" {
  name     = var.db_user.username
  instance = google_sql_database_instance.default.name
  password = data.google_secret_manager_secret_version.db_password.secret_data
}


resource "google_service_account" "frontend" {
  account_id   = var.frontend.service_name
  display_name = "Chronicle front-end"
}

resource "google_secret_manager_secret" "backend_url" {
  secret_id = "${var.backend.service_name}-url"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "backend_url_placeholder" {
  secret      = google_secret_manager_secret.backend_url.id
  secret_data = " "
}

resource "google_secret_manager_secret_iam_member" "backend_url" {
  secret_id = google_secret_manager_secret.backend_url.secret_id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.frontend.email}"
}

resource "google_cloud_run_v2_service" "frontend" {
  name                = var.frontend.service_name
  location            = var.region
  ingress             = "INGRESS_TRAFFIC_ALL"
  deletion_protection = false
  template {
    service_account       = google_service_account.frontend.email
    execution_environment = "EXECUTION_ENVIRONMENT_GEN2"
    containers {
      image = var.frontend.image_url
      resources {
        limits = {
          memory = "2Gi"
          cpu    = "4"
        }
        cpu_idle = true
      }
      env {
        name = "PUBLIC_API_URL"
        value_source {
          secret_key_ref {
            secret  = google_secret_manager_secret.backend_url.secret_id
            version = "latest"
          }
        }
      }
    }
  }
}

resource "google_cloud_run_v2_service_iam_member" "public_frontend" {
  name     = google_cloud_run_v2_service.frontend.name
  location = google_cloud_run_v2_service.frontend.location
  role     = "roles/run.invoker"
  member   = "allUsers"
}


resource "google_service_account" "backend" {
  account_id   = var.backend.service_name
  display_name = "Chronicle back-end"
}

resource "google_project_iam_member" "cloudsql" {
  project = var.project_id
  role    = "roles/cloudsql.client"
  member  = "serviceAccount:${google_service_account.backend.email}"
}

resource "google_secret_manager_secret_iam_member" "db_password" {
  secret_id = var.db_user.password_secret_id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.backend.email}"
}

resource "google_secret_manager_secret_iam_member" "admin_password" {
  secret_id = var.admin.password_secret_id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.backend.email}"
}

resource "google_secret_manager_secret_iam_member" "session_key" {
  secret_id = var.session_key_secret_id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.backend.email}"
}

resource "google_cloud_run_v2_service" "backend" {
  name                = var.backend.service_name
  location            = var.region
  ingress             = "INGRESS_TRAFFIC_ALL"
  deletion_protection = false
  template {
    service_account       = google_service_account.backend.email
    execution_environment = "EXECUTION_ENVIRONMENT_GEN2"
    containers {
      image = var.backend.image_url
      resources {
        limits = {
          memory = "2Gi"
          cpu    = "4"
        }
        cpu_idle = true
      }
      env {
        name  = "APP__PORT"
        value = 8080
      }
      env {
        name  = "APP__ALLOWED_ORIGIN"
        value = join(",", google_cloud_run_v2_service.frontend.urls)
      }
      env {
        name = "APP__SESSION_KEY"
        value_source {
          secret_key_ref {
            secret  = var.session_key_secret_id
            version = "latest"
          }
        }
      }
      env {
        name  = "APP__ADMIN__USERNAME"
        value = var.admin.username
      }
      env {
        name = "APP__ADMIN__PASSWORD"
        value_source {
          secret_key_ref {
            secret  = var.admin.password_secret_id
            version = "latest"
          }
        }
      }
      env {
        name  = "APP__DATABASE__HOST"
        value = google_sql_database_instance.default.private_ip_address
      }
      env {
        name  = "APP__DATABASE__NAME"
        value = google_sql_database.default.name
      }
      env {
        name  = "APP__DATABASE__USERNAME"
        value = google_sql_user.default.name
      }
      env {
        name = "APP__DATABASE__PASSWORD"
        value_source {
          secret_key_ref {
            secret  = var.db_user.password_secret_id
            version = "latest"
          }
        }
      }
    }
    vpc_access {
      network_interfaces {
        network = "default"
      }
    }
  }
}

resource "google_secret_manager_secret_version" "backend_url" {
  secret      = google_secret_manager_secret.backend_url.id
  secret_data = google_cloud_run_v2_service.backend.uri
  lifecycle {
    create_before_destroy = true
  }
}

resource "google_cloud_run_v2_service_iam_member" "public_backend" {
  name     = google_cloud_run_v2_service.backend.name
  location = google_cloud_run_v2_service.backend.location
  role     = "roles/run.invoker"
  member   = "allUsers"
}
