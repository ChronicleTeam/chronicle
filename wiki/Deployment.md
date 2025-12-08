# Deployment

Chronicle is built to be deployed on Google Cloud Platform via Terraform. Chronicle deploys two public services and an private database.

## Components

### Database

File: `terraform/database.tf`

The Chronicle database is a [Cloud SQL](https://docs.cloud.google.com/sql/docs/postgres/features) instance using PostgreSQL 17. The database is created along with its credentials. The instance is exposed only inside the default [VPC](https://docs.cloud.google.com/vpc/docs/overview).


### Backend

File: `terraform/backend.tf`

The backend is a [Cloud Run](https://docs.cloud.google.com/run/docs/overview/what-is-cloud-run) service that serves backend HTTP requests. It runs the Rust Axum web server as a container image. The service is given the database connection info and acces to the default VPC to connect to the database. It is exposed publicly to allow the frontend to access it.

The CI/CD uses [Cloud Build](https://docs.cloud.google.com/build/docs/overview) triggers to run tests on pull requests, and build then push the container image on merges to the main branch.

### Frontend

File: `terraform/frontend.tf`

The frontend is also a [Cloud Run](https://docs.cloud.google.com/run/docs/overview/what-is-cloud-run) service that serves HTTP requests. It runs the Typescript Svelte application as a container image. It is exposed to the public.

The CI/CD uses [Cloud Build](https://docs.cloud.google.com/build/docs/overview) triggers to run tests on pull requests, and build then push the container image on merges to the main branch.

## Client Handover

For the client handover, we will transfer this repository over to the client's organization GitHub, and leave instructions for [installation](Installation.md). The Terraform files and the instructions should provide everything the client needs to deploy the application into a GCP environment.