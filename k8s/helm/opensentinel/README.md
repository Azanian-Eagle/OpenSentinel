# OpenSentinel Helm Chart

Official Helm Chart for **OpenSentinel**, engineered by **Azanian Eagle** to enable zero-downtime, enterprise-grade Kubernetes deployments of the OpenSentinel non-invasive bot-detection engine.

## About OpenSentinel & Azanian Eagle

Developed by [Azanian Eagle](https://github.com/Azanian-Eagle), OpenSentinel is built on the core principle of **digital sovereignty** and **user privacy**. Traditional CAPTCHA systems force users to solve image puzzles, collecting invasive tracking profiles and training commercial third-party AI models.

OpenSentinel replaces intrusive visual challenges with frictionless client telemetry analysis. Operating locally on your Kubernetes infrastructure, OpenSentinel evaluates abstract mathematical parameters in cursor trajectories and keystroke dynamics to detect automated bots without collecting personally identifiable information (PII).

### Key Helm Features
- **Zero-Downtime Deployment:** Rolling update strategy ensuring uninterrupted verification services.
- **Horizontal Pod Autoscaling (HPA):** Built-in target CPU and Memory utilization autoscaling (default 3 to 10 replicas).
- **ExternalSecrets Operator Integration:** Optional HashiCorp Vault, AWS Secrets Manager, or Azure Key Vault secret management via `ExternalSecret` resource templates.
- **Prometheus Operator Support:** Auto-configured `ServiceMonitor` templates for seamless metrics scraping.
- **Resource Management:** Pre-configured CPU/memory requests and limits designed for cloud efficiency.

---

## Prerequisites

- **Kubernetes:** Cluster version `1.20+`
- **Helm:** Version `3.8.0+`
- **Container Registry Access:** Access to `ghcr.io/azanian-eagle/opensentinel`

---

## Quickstart Installation

### 1. Install via OCI Registry (GHCR)

```bash
helm install opensentinel oci://ghcr.io/azanian-eagle/helm/opensentinel \
  --version 1.0.4 \
  --set env.payloadSecretKey="0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"
```

### 2. Install from Local Chart Directory

```bash
git clone https://github.com/Azanian-Eagle/OpenSentinel.git
cd OpenSentinel

helm install opensentinel k8s/helm/opensentinel \
  --set env.payloadSecretKey="0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"
```

---

## Configuration Parameter Reference (`values.yaml`)

The following table lists the configurable parameters of the OpenSentinel chart and their default values:

| Parameter | Description | Default Value |
|---|---|---|
| `replicaCount` | Initial number of backend pod replicas | `3` |
| `image.repository` | Container image repository | `ghcr.io/azanian-eagle/opensentinel` |
| `image.tag` | Container image tag | `"latest"` |
| `image.pullPolicy` | Image pull policy | `Always` |
| `service.type` | Kubernetes Service type (`ClusterIP`, `NodePort`, `LoadBalancer`) | `ClusterIP` |
| `service.port` | Service network listening port | `8080` |
| `resources.limits.cpu` | CPU resource limit per pod | `200m` |
| `resources.limits.memory` | Memory resource limit per pod | `256Mi` |
| `resources.requests.cpu` | Minimum requested CPU per pod | `100m` |
| `resources.requests.memory` | Minimum requested memory per pod | `128Mi` |
| `autoscaling.enabled` | Enable HorizontalPodAutoscaler (HPA) | `true` |
| `autoscaling.minReplicas` | Minimum HPA pod count | `3` |
| `autoscaling.maxReplicas` | Maximum HPA pod count | `10` |
| `autoscaling.targetCPUUtilizationPercentage` | HPA target CPU utilization percentage | `80` |
| `autoscaling.targetMemoryUtilizationPercentage` | HPA target Memory utilization percentage | `80` |
| `secretName` | Name of Kubernetes Secret storing `PAYLOAD_SECRET_KEY` | `opensentinel-secrets` |
| `externalSecrets.enabled` | Enable integration with Kubernetes ExternalSecrets operator | `false` |
| `externalSecrets.secretStoreRef.name` | External Secrets store reference name | `"aws-secretsmanager"` |
| `externalSecrets.secretStoreRef.kind` | External Secrets store reference kind | `"ClusterSecretStore"` |

---

## Production Security & External Secrets

For enterprise production environments, we recommend populating secrets via External Secrets Operator rather than static values:

```yaml
externalSecrets:
  enabled: true
  secretStoreRef:
    name: "aws-secretsmanager"
    kind: "ClusterSecretStore"
  targetSecretName: "opensentinel-external-secrets"
  data:
    - secretKey: PAYLOAD_SECRET_KEY
      remoteRef:
        key: "opensentinel/payload-secret-key"
```

---

## Regulatory Compliance & Digital Sovereignty

OpenSentinel is engineered by **Azanian Eagle** to comply with global data protection standards:
- **POPIA (South Africa):** Guarantees data privacy compliance by running local machine learning models without collecting personal identifiers.
- **GDPR & CCPA:** Ensures complete digital sovereignty with zero third-party data tracking.

---

## Licence

Distributed under the **MIT Licence**. Engineered with pride by **Azanian Eagle**.
