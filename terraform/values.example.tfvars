project_id = "<project_id>"
region = "<region>"
network_name = "default"

production_db = {
    instance_name = "chronicle"
    name = "chronicle"
    username = "chronicle"
    password_secret_id = "chronicle-db-password"
}

backend = {
  service_name = "chronicle-backend"
  image_url = "<image_repository>/backend:latest"
}

frontend = {
  service_name = "chronicle"
  image_url = "<image_repository>/frontend:latest"
  urls_secret_id = "frontend-urls"
}

admin = {
  username = "<admin_email>"
  password_secret_id = "chronicle-admin-password"
}

session_key_secret_id = "chronicle-session-key"

github = {
  username = "<github_username>"
  repo = "chronicle"
}