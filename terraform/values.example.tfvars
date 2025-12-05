project_id = "project-id"
region = "northamerica-northeast1"
network_name = "default"

production_db = {
    instance_name = "chronicle"
    name = "chronicle"
    username = "chronicle"
    password_secret_id = "chronicle-db-password"
}

test_db = {
    instance_name = "chronicle-test"
    name = "chronicle-test"
    username = "chronicle"
    password_secret_id = "chronicle-test-db-password"
}

backend = {
  service_name = "chronicle-backend"
  image_url = "northamerica-northeast1-docker.pkg.dev/project-id/chronicle/backend:latest"
#   allowed_origin = [ "https://chronicle-XXXXXXX.run.app", "https://chronicle-ABCD.a.run.app" ]

}

frontend = {
  service_name = "chronicle"
  image_url = "northamerica-northeast1-docker.pkg.dev/project-id/chronicle/frontend:latest"
  urls_secret_id = "frontend-urls"
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