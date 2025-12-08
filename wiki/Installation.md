# Installation

## Local environment

### Dependencies:

#### Rust
https://www.rust-lang.org/tools/install

#### Node.js
https://nodejs.org/en/download

#### Podman
https://podman.io/docs/installation

Altenatively, Docker can work as a replacement

https://docs.docker.com/engine/install/

### Backend
From project root:
```
cd backend
```

Setup environment variables:
```
cp example.env .env
```

Run database
```
podman compose up -d
```

Run web server:
```
cargo run --release
```

### Frontend
From project root:
```
cd frontend
```

```
echo 'PUBLIC_API_URL=http://localhost:5000' > .env
```

```
npm dev run
```

### Open Application

OpenAPI documentation: http://localhost:5000/docs

Application: http://localhost:5173

#### Login: 

email: `test@example.com`

password: `test123`

### View application

Visit `http://localhost:5000/docs`

## Google Cloud Platform
From project root:
```
cd terraform
```

### Dependencies:

#### Google Cloud CLI:
https://docs.cloud.google.com/sdk/docs/install-sdk

#### Terraform
https://developer.hashicorp.com/terraform/install

### Create secrets

In the GCP console, go to **Secret Manager** and create the following secrets:

`chronicle-admin-password`

Password of the first admin user. Create a long and random password.

`chronicle-db-password`

Password of the system's Cloud SQL database. Create a long and random password.

`chronicle-session-key`

Cryptographically secure key used to sign session cookies. Must be base64 encoded and at least 64 bytes.

The following command may be used to generate it:
```
head -c 256 /dev/urandom | base64 -w 0
```

`frontend-urls`

Set this to a single whitespace

### Create image repository

Got to **Artifacts Registry** and create a repository named `chronicle` with Docker format.

Go to **Cloud Build** and then **Repositories** and connect the Chronicle GitHub repository using 1st gen.

Run this command to build the initial images.
```
bash scripts/build.sh <chronicle_github> <image_repository>
```

This may take several minutes.

### Initialize Terraform

Search **Cloud Storage** and create a bucket in which to store the terraform state. It is recommended to enable versioning.

Copy template terraform backend:
```
cp backend.example.hcl backend.hcl
```

Replace `<tfstate-bucket>` in `backend.hcl` with your bucket name.

Initialize Terraform
```
terraform init --backend-config backend.hcl
```

### Apply Terraform

Copy template variables:
```
cp values.example.tfvars values.tfvars
```

Replace All the values inside angle brackets (`<value_name>`). Other values may be changed as needed.

Apply Terraform
```
terraform apply --var-file values.tfvars
```

This may take several minutes.

The output will show the application URLs. Copy these to replace the `backend.allowed_origin` value. For example:
```
backend = {
  ...
  allowed_origin = [
    "https://chronicle-XXXXXXXX.run.app",
    "https://chronicle-ABCD.a.run.app",
  ]
}
```

Then re-apply the Terraform
```
terraform apply --var-file values.tfvars
```

### View services

**Cloud Run Services**: Frontend and backend services

**Cloud SQL**: Chronicle database

**Cloud Build Triggers**: CI/CD

### View application

Go to **Cloud Run** and select the frontend service to view the application URL. Select the backend service and append `/docs` to the URL to view the OpenAPI documentation.

#### Login

email: Previously set in `values.tfvars`

password: Go to **Secret Manager** to view the admin password

