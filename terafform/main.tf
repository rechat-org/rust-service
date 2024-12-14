resource "digitalocean_kubernetes_cluster" "chat_cluster" {
  name    = "chat-cluster"
  region  = var.region
  version = "1.31.1-do.5"

  node_pool {
    name       = "worker-pool"
    size       = "s-2vcpu-4gb"
    node_count = 2
    auto_scale = true
    min_nodes  = 2
    max_nodes  = 5
  }
}

# Store kube config
resource "local_file" "kubeconfig" {
  content  = digitalocean_kubernetes_cluster.chat_cluster.kube_config[0].raw_config
  filename = "${path.module}/kubeconfig"
}

# Create Container Registry
resource "digitalocean_container_registry" "chat_registry" {
  name                   = "chat-service"
  subscription_tier_slug = "basic"
}

# Grant Registry Access to K8s Cluster
resource "digitalocean_container_registry_docker_credentials" "chat_registry_credentials" {
  registry_name = digitalocean_container_registry.chat_registry.name
}

# Add output for the registry URL
output "registry_url" {
  value = digitalocean_container_registry.chat_registry.server_url
}

output "registry_name" {
  value = digitalocean_container_registry.chat_registry.name
}

# Create a PostgreSQL database cluster
resource "digitalocean_database_cluster" "postgres_cluster" {
  name       = "chat-postgres-cluster"
  engine     = "pg"
  version    = "14"
  size = "db-s-1vcpu-1gb"  # Smallest size for dev
  region     = var.region
  node_count = 1  # Single node for dev
}

# Create a database
resource "digitalocean_database_db" "chat_database" {
  cluster_id = digitalocean_database_cluster.postgres_cluster.id
  name       = "chat_api"
}

# Create a database user
resource "digitalocean_database_user" "chat_user" {
  cluster_id = digitalocean_database_cluster.postgres_cluster.id
  name       = "chat_user"
}

# Add outputs to get connection info
output "database_host" {
  value = digitalocean_database_cluster.postgres_cluster.host
}

output "database_port" {
  value = digitalocean_database_cluster.postgres_cluster.port
}

output "database_user" {
  value = digitalocean_database_user.chat_user.name
}

output "database_password" {
  value     = digitalocean_database_user.chat_user.password
  sensitive = true
}

output "database_url" {
  value     = "postgres://${digitalocean_database_user.chat_user.name}:${digitalocean_database_user.chat_user.password}@${digitalocean_database_cluster.postgres_cluster.host}:${digitalocean_database_cluster.postgres_cluster.port}/chat_api"
  sensitive = true
}