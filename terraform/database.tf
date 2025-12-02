
resource "google_compute_global_address" "production_db" {
  name          = "${var.production_db.instance_name}-db"
  purpose       = "VPC_PEERING"
  address_type  = "INTERNAL"
  prefix_length = 24
  network       = data.google_compute_network.default.id
}

resource "google_service_networking_connection" "production_db" {
  network                 = data.google_compute_network.default.id
  service                 = "servicenetworking.googleapis.com"
  reserved_peering_ranges = [google_compute_global_address.production_db.name]
}

resource "google_sql_database_instance" "production_db" {
  name                = var.production_db.instance_name
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
  depends_on = [google_service_networking_connection.production_db]
}

resource "google_sql_database" "production_db" {
  name     = var.production_db.name
  instance = google_sql_database_instance.production_db.name
}


data "google_secret_manager_secret_version" "production_db_password" {
  secret  = var.production_db.password_secret_id
  version = "latest"
}

resource "google_sql_user" "production_db" {
  name     = var.production_db.username
  instance = google_sql_database_instance.production_db.name
  password = data.google_secret_manager_secret_version.production_db_password.secret_data
}



resource "google_compute_global_address" "test_db" {
  name          = "${var.test_db.instance_name}-db"
  purpose       = "VPC_PEERING"
  address_type  = "INTERNAL"
  prefix_length = 24
  network       = data.google_compute_network.default.id
}

resource "google_service_networking_connection" "test_db" {
  network                 = data.google_compute_network.default.id
  service                 = "servicenetworking.googleapis.com"
  reserved_peering_ranges = [google_compute_global_address.test_db.name]
}

resource "google_sql_database_instance" "test_db" {
  name                = var.test_db.instance_name
  region              = var.region
  database_version    = "POSTGRES_17"
  deletion_protection = false
  settings {
    edition = "ENTERPRISE"
    tier    = "db-f1-micro"
    ip_configuration {
      ipv4_enabled = true
    }
    # database_flags {
    #   name  = "idle_in_transaction_session_timeout"
    #   value = "10000" # 10 seconds (aggressive for tests)
    # }
    # database_flags {
    #   name  = "tcp_keepalives_idle"
    #   value = "30" # 30 seconds (more aggressive)
    # }
    # database_flags {
    #   name  = "tcp_keepalives_interval"
    #   value = "10"
    # }
    # database_flags {
    #   name  = "max_connections"
    #   value = "20" # Lower than default
    # }
  }
  depends_on = [google_service_networking_connection.test_db]
}

resource "google_sql_database" "test_db" {
  name     = var.test_db.name
  instance = google_sql_database_instance.test_db.name
}


data "google_secret_manager_secret_version" "test_db_password" {
  secret  = var.test_db.password_secret_id
  version = "latest"
}

resource "google_sql_user" "test_db" {
  name     = var.test_db.username
  instance = google_sql_database_instance.test_db.name
  password = data.google_secret_manager_secret_version.test_db_password.secret_data
}


