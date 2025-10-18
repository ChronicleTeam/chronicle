
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

resource "google_cloudbuild_trigger" "frontend" {
  name            = "${var.frontend.service_name}-trigger"
  service_account = google_service_account.frontend_build.id
  github {
    owner = var.github.username
    name  = var.github.repo
    push {
      branch = "main"
    }
  }
  included_files = ["frontend/**"]
  substitutions = {
    _IMAGE_URL    = var.frontend.image_url
    _SERVICE_NAME = var.frontend.service_name
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
      args = ["build", "-t", "$_IMAGE_URL:$COMMIT_SHA", "frontend"]
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

resource "google_service_account" "frontend_build" {
  account_id   = "${var.frontend.service_name}-build"
  display_name = "Chronicle back-end"
}

resource "google_project_iam_member" "frontend_act_as" {
  project = var.project_id
  role    = "roles/iam.serviceAccountUser"
  member  = "serviceAccount:${google_service_account.frontend_build.email}"
}

resource "google_project_iam_member" "frontend_logs_writer" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.frontend_build.email}"
}

resource "google_project_iam_member" "frontend_artifact_registry_writer" {
  project = var.project_id
  role    = "roles/artifactregistry.writer"
  member  = "serviceAccount:${google_service_account.frontend_build.email}"
}

resource "google_project_iam_member" "frontend_cloud_run_admin" {
  project = var.project_id
  role    = "roles/run.admin"
  member  = "serviceAccount:${google_service_account.frontend_build.email}"
}
