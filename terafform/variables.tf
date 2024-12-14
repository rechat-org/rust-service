variable "do_token" {
  description = "Digital Ocean API Token"
  sensitive = true
}

variable "region" {
  description = "Digital Ocean region"
  default = "nyc1"
}