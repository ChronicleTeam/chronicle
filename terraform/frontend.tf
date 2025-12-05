
resource "google_cloud_run_v2_service" "frontend" {
  name                = var.frontend.service_name
  location            = var.region
  ingress             = "INGRESS_TRAFFIC_ALL"
  deletion_protection = false
  template {
    service_account       = google_service_account.frontend.email
    execution_environment = "EXECUTION_ENVIRONMENT_GEN2"
    containers {
      image = "${var.frontend.image_url}:latest"
      resources {
        limits = {
          memory = "2Gi"
          cpu    = "4"
        }
        cpu_idle = true
      }
      env {
        name  = "PUBLIC_API_URL"
        value = google_cloud_run_v2_service.backend.uri
      }
    }
  }
}

output "frontend_urls" {
  value = google_cloud_run_v2_service.frontend.urls
}

resource "google_service_account" "frontend" {
  account_id   = var.frontend.service_name
  display_name = "Chronicle frontend"
}

resource "google_cloud_run_v2_service_iam_member" "frontend_public" {
  name     = google_cloud_run_v2_service.frontend.name
  location = google_cloud_run_v2_service.frontend.location
  role     = "roles/run.invoker"
  member   = "allUsers"
}

resource "google_cloudbuild_trigger" "frontend_ci" {
  name            = "${var.frontend.service_name}-ci"
  service_account = google_service_account.frontend_ci.id

  github {
    owner = var.github.username
    name  = var.github.repo

    pull_request {
      branch = ".*"
    }
  }

  included_files = ["frontend/**"]
  filename       = "terraform/cloudbuild/frontend.ci.yaml"
}

resource "google_service_account" "frontend_ci" {
  account_id   = "${var.frontend.service_name}-ci"
  display_name = "Chronicle frontend CI"
}

resource "google_project_iam_member" "frontend_ci_log_writer" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.frontend_ci.email}"
}

resource "google_cloudbuild_trigger" "frontend_cd" {
  name            = "${var.frontend.service_name}-cd"
  service_account = google_service_account.frontend_cd.id
  github {
    owner = var.github.username
    name  = var.github.repo
    push {
      branch = "main"
    }
  }
  included_files = ["frontend/**"]
  substitutions = {
    _DIRECTORY    = "frontend"
    _IMAGE_URL    = var.frontend.image_url
    _SERVICE_NAME = var.frontend.service_name
    _REGION       = var.region
  }
  filename = "terraform/cloudbuild/cd.yaml"
}

resource "google_service_account" "frontend_cd" {
  account_id   = "${var.frontend.service_name}-cd"
  display_name = "Chronicle frontend CD"
}

resource "google_project_iam_member" "frontend_cd_act_as" {
  project = var.project_id
  role    = "roles/iam.serviceAccountUser"
  member  = "serviceAccount:${google_service_account.frontend_cd.email}"
}

resource "google_project_iam_member" "frontend_cd_logs_writer" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.frontend_cd.email}"
}

resource "google_project_iam_member" "frontend_cd_artifact_registry_writer" {
  project = var.project_id
  role    = "roles/artifactregistry.writer"
  member  = "serviceAccount:${google_service_account.frontend_cd.email}"
}

resource "google_project_iam_member" "frontend_cd_cloud_run_admin" {
  project = var.project_id
  role    = "roles/run.admin"
  member  = "serviceAccount:${google_service_account.frontend_cd.email}"
}
