
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


resource "google_cloudbuild_trigger" "backend" {
  name            = "${var.backend.service_name}-trigger"
  service_account = google_service_account.backend_build.id
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
  build {
    timeout = "3600s"
    options {
      logging      = "CLOUD_LOGGING_ONLY"
      machine_type = "E2_HIGHCPU_8"
    }
    step {
      name = "gcr.io/cloud-builders/docker"
      args = ["build", "-t", "$_IMAGE_URL:$COMMIT_SHA", "."]
    }
    step {
      name = "gcr.io/cloud-builders/docker"
      args = ["push", "$_IMAGE_URL:$COMMIT_SHA"]
    }
    step {
      name       = "gcr.io/google.com/cloudsdktool/cloud-sdk"
      entrypoint = "gcloud"
      args = [
        "run", "deploy", "$_SERVICE_NAME",
        "--image", "$_IMAGE_URL:$COMMIT_SHA",
        "--region", "$_REGION",
        "--platform", "managed",
        "--quiet"
      ]
    }
  }
}

resource "google_service_account" "backend_build" {
  account_id   = "${var.backend.service_name}-build"
  display_name = "Chronicle back-end"
}

resource "google_project_iam_member" "backend_act_as" {
  project = var.project_id
  role    = "roles/iam.serviceAccountUser"
  member  = "serviceAccount:${google_service_account.backend_build.email}"
}

resource "google_project_iam_member" "backend_logs_writer" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.backend_build.email}"
}

resource "google_project_iam_member" "backend_artifact_registry_writer" {
  project = var.project_id
  role    = "roles/artifactregistry.writer"
  member  = "serviceAccount:${google_service_account.backend_build.email}"
}

resource "google_project_iam_member" "backend_cloud_run_admin" {
  project = var.project_id
  role    = "roles/run.admin"
  member  = "serviceAccount:${google_service_account.backend_build.email}"
}
