set -e -o pipefail

IMAGE=northamerica-northeast1-docker.pkg.dev/basic-bank-471814-c7/chronicle/frontend:latest

gcloud auth configure-docker northamerica-northeast1-docker.pkg.dev

docker build -t $IMAGE .
docker push $IMAGE
