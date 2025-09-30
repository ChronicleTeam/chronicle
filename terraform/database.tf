
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
