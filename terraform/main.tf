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
      private_network = data.google_compute_network.default.id
      # enable_private_path_for_google_cloud_services = true
    }
  }
}

resource "google_sql_database" "default" {
  name     = var.db_name
  instance = google_sql_database_instance.default.name
}

resource "google_sql_user" "default" {
  name     = var.db_user.username
  instance = google_sql_database_instance.default.name
  password = var.db_user.password
}

# resource "google_vpc_access_connector" "default" {
#   name          =  var.db_instance_name
#   network       = data.google_compute_network.default.self_link
#   region        = "us-central1"
#   ip_cidr_range = "10.8.0.0/28"
# }

resource "google_service_account" "backend" {
  account_id   = var.backend.service_name
  display_name = "Chronicle back-end"
}

resource "google_project_iam_member" "cloudsql" {
  project = var.project_id
  role    = "roles/cloudsql.client"
  member  = "serviceAccount:${google_service_account.backend.email}"
}


resource "google_cloud_run_v2_service" "default" {
  name                = var.backend.service_name
  location            = var.region
  ingress             = "INGRESS_TRAFFIC_ALL"
  deletion_protection = false
  template {
    service_account       = google_service_account.backend.email
    execution_environment = "EXECUTION_ENVIRONMENT_GEN2"
    containers {
      image = "${var.backend.image_url}:latest"
      resources {
        limits = {
          memory = "2Gi"
          cpu    = "4"
        }
        cpu_idle = true
      }
      env {
        name  = "APP_PORT"
        value = 8080
      }
      env {
        name  = "APP_ALLOWED_ORIGIN"
        value = ""
      }
      env {
        name = "APP_SESSION_KEY"
        value_source {
          secret_key_ref {
            secret = var.session_key_secret_id
          }
        }
      }
      env {
        name  = "APP_DATABASE_HOST"
        value = ""
      }
      env {
        name  = "APP_DATABASE_NAME"
        value = ""
      }
      env {
        name  = "APP_DATABASE_USERNAME"
        value = var.db_user.username
      }
      env {
        name = "APP_DATABASE_PASSWORD"
        value_source {
          secret_key_ref {
            secret = var.db_user.password_secret_id
          }
        }
      }
    }
    vpc_access {
      network_interfaces {
        network    = "default"
        subnetwork = "default"
        # tags       = ["tag1", "tag2", "tag3"]
      }
    }
  }
}
