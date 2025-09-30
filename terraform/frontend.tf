
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
