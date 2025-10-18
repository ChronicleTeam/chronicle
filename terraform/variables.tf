
variable "project_id" {}
variable "region" {}

variable "network_name" {}

variable "db_instance_name" {}
variable "db_name" {}
variable "db_user" {
  type = object({
    username           = string
    password_secret_id = string
  })
}

variable "backend" {
  type = object({
    service_name = string
    image_url    = string
  })
}

variable "frontend" {
  type = object({
    service_name = string
    image_url    = string
  })
}

variable "admin" {
  type = object({
    username           = string
    password_secret_id = string
  })
}
variable "session_key_secret_id" {}

variable "github" {
  type = object({
    username = string
    repo     = string
  })
}
