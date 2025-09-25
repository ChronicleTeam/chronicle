project_id = "project-id"
region = "northamerica-northeast1"
network_name = "default"

db_instance_name = "chronicle"
db_name = "chronicle"
db_user = {
  username = "chronicle"
  password_secret_id = "chronicle-db-password"
}

backend = {
  service_name = "chronicle-backend"
  image_url = "northamerica-northeast1-docker.pkg.dev/project-id/chronicle/backend:latest"
}

frontend = {
  service_name = "chronicle"
  image_url = "northamerica-northeast1-docker.pkg.dev/project-id/chronicle/frontend:latest"
}

admin = {
  username = "admin_username"
  password_secret_id = "chronicle-admin-password"
}

session_key_secret_id = "chronicle-session-key"

github = {
  username = "ChronicleTeam"
  repo = "chronicle"
}