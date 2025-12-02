
resource "google_service_account" "backend" {
  account_id   = var.backend.service_name
  display_name = "Chronicle backend"
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

resource "google_cloudbuild_trigger" "backend_ci" {
  name            = "${var.backend.service_name}-ci"
  service_account = google_service_account.backend_ci.id

  github {
    owner = var.github.username
    name  = var.github.repo

    pull_request {
      branch = ".*"
    }
  }

  included_files = ["backend/**"]

  filename = "terraform/cloudbuild/backend.ci.yaml"
}

resource "google_service_account" "backend_ci" {
  account_id   = "${var.backend.service_name}-ci"
  display_name = "Chronicle backend CI"
}


resource "google_project_iam_member" "backend_ci_log_writer" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.backend_ci.email}"
}

resource "google_cloudbuild_trigger" "backend_cd" {
  name            = "${var.backend.service_name}-cd"
  service_account = google_service_account.backend_cd.id
  github {
    owner = var.github.username
    name  = var.github.repo
    push {
      branch = "main"
    }
  }
  included_files = ["backend/**"]
  substitutions = {
    _IMAGE_URL    = var.backend.image_url
    _SERVICE_NAME = var.backend.service_name
    _REGION       = var.region
  }
  filename = "terraform/cloudbuild/backend.cd.yaml"
}

resource "google_service_account" "backend_cd" {
  account_id   = "${var.backend.service_name}-cd"
  display_name = "Chronicle backend CD"
}

resource "google_project_iam_member" "backend_cd_act_as" {
  project = var.project_id
  role    = "roles/iam.serviceAccountUser"
  member  = "serviceAccount:${google_service_account.backend_cd.email}"
}

resource "google_project_iam_member" "backend_cd_log_writer" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.backend_cd.email}"
}

resource "google_project_iam_member" "backend_cd_artifact_registry_writer" {
  project = var.project_id
  role    = "roles/artifactregistry.writer"
  member  = "serviceAccount:${google_service_account.backend_cd.email}"
}

resource "google_project_iam_member" "backend_cd_run_admin" {
  project = var.project_id
  role    = "roles/run.admin"
  member  = "serviceAccount:${google_service_account.backend_cd.email}"
}
