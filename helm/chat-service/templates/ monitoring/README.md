# Chat Service Monitoring Setup

This document describes the process of setting up monitoring for the chat service using Promtail, Loki, and Grafana in a Kubernetes environment.

## Overview

The monitoring stack consists of:
- Promtail (log collector) running as a sidecar container
- Loki (log aggregation) in a separate monitoring namespace
- Grafana (visualization) in the monitoring namespace

## Prerequisites

- Kubernetes cluster with Helm installed
- `kubectl` configured to access your cluster
- Helm v3.x or later
- Access to DigitalOcean container registry

## Step 1: Create Monitoring Namespace

```bash
kubectl create namespace monitoring
```

## Step 2: Install Monitoring Stack

Add the Grafana Helm repository and install the monitoring stack:

```bash
# Add Grafana helm repo
helm repo add grafana https://grafana.github.io/helm-charts
helm repo update

# Install Loki stack (includes Grafana and Prometheus)
helm install loki grafana/loki-stack \
  --namespace monitoring \
  --set grafana.enabled=true \
  --set prometheus.enabled=true \
  --create-namespace
```

## Step 3: Configure Promtail

### Add Promtail Configuration to Values

Add the following to your `values.yaml`:

```yaml
monitoring:
  enabled: true
  namespace: monitoring
  promtail:
    image:
      repository: grafana/promtail
      tag: "2.9.1"
      pullPolicy: IfNotPresent
    resources:
      limits:
        cpu: 200m
        memory: 128Mi
      requests:
        cpu: 100m
        memory: 64Mi
  loki:
    enabled: true
    url: http://loki.monitoring.svc.cluster.local:3100
  grafana:
    enabled: true
  prometheus:
    enabled: true
```

### Create Promtail ConfigMap

Create `templates/promtail-configmap.yaml`:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ include "chat-service.fullname" . }}-promtail
data:
  promtail.yaml: |
    server:
      http_listen_port: 9080
    positions:
      filename: /tmp/positions.yaml
    clients:
      - url: http://{{ .Values.monitoring.loki.url }}/loki/api/v1/push
    scrape_configs:
      - job_name: chat-service-logs
        static_configs:
          - targets:
              - localhost
            labels:
              job: chat-service
              __path__: /var/log/*.log
        pipeline_stages:
          - json:
              expressions:
                timestamp: time
                level: level
                message: msg
```

## Step 4: Update Deployment

Modify your deployment to include the Promtail sidecar container:

```yaml
spec:
  template:
    spec:
      volumes:
        - name: shared-logs
          emptyDir: {}
        - name: promtail-config
          configMap:
            name: {{ include "chat-service.fullname" . }}-promtail
      containers:
        # Your main container config...
        {{- if and .Values.monitoring.enabled .Values.monitoring.promtail }}
        - name: promtail
          image: "{{ .Values.monitoring.promtail.image.repository }}:{{ .Values.monitoring.promtail.image.tag }}"
          args:
            - -config.file=/etc/promtail/promtail.yaml
          volumeMounts:
            - name: shared-logs
              mountPath: /var/log
            - name: promtail-config
              mountPath: /etc/promtail
          ports:
            - containerPort: 9080
              name: http-metrics
          resources:
            {{- toYaml .Values.monitoring.promtail.resources | nindent 12 }}
        {{- end }}
```

## Step 5: RBAC Setup

Create `templates/rbac.yaml`:

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: {{ include "chat-service.fullname" . }}
  labels:
    {{- include "chat-service.labels" . | nindent 4 }}
imagePullSecrets:
  - name: chat-service-registry
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: {{ include "chat-service.fullname" . }}
rules:
  - apiGroups: [""]
    resources: ["pods"]
    verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: {{ include "chat-service.fullname" . }}
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: {{ include "chat-service.fullname" . }}
subjects:
  - kind: ServiceAccount
    name: {{ include "chat-service.fullname" . }}
    namespace: {{ .Release.Namespace }}
```

## Step 6: Registry Access Setup

Create the registry secret for pulling images:

```bash
kubectl create secret docker-registry chat-service-registry \
  --docker-server=registry.digitalocean.com \
  --docker-username=<your-do-token> \
  --docker-password=<your-do-token>
```

## Step 7: Deploy

Deploy using Helm:

```bash
helm upgrade --install chat-service ./helm/chat-service \
  --set image.tag=<your-image-tag> \
  --wait --timeout 10m
```

## Verification

1. Check if pods are running:
```bash
kubectl get pods
```

2. Check Promtail logs:
```bash
kubectl logs <pod-name> -c promtail
```

3. Access Grafana:
```bash
kubectl port-forward svc/loki-grafana 3000:80 -n monitoring
```

Default Grafana credentials:
- Username: admin
- Password: Get using: `kubectl get secret loki-grafana -n monitoring -o jsonpath="{.data.admin-password}" | base64 --decode`

## Common Issues and Troubleshooting

1. If pods can't pull images:
   - Verify registry secret exists and is correctly referenced
   - Check DO token permissions
   - Verify image tags are correct

2. If Promtail can't connect to Loki:
   - Verify Loki service is running in monitoring namespace
   - Check Promtail configuration URL
   - Verify network policies allow communication

3. If logs aren't appearing in Grafana:
   - Check Promtail configuration
   - Verify log file paths
   - Check Promtail sidecar logs for errors

## Maintenance

- Monitor resource usage of Promtail sidecar
- Regularly update Promtail image version
- Review and adjust resource limits as needed
- Periodically rotate DO registry credentials