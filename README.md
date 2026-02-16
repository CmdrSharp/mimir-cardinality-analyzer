# Mimir Cardinality Analyzer

When running Mimir at scale, you inevitably wind up accumulating metrics that are completely unused. No dashboard references them and they're not used for alerting. These metrics however still consume resources both in terms of storage and ingest-capacity. In Grafana Cloud, you have Adaptive Metrics to surface this problem, but nothing similar exists for on-premises deployments. This tool identifies unused metrics in Mimir by cross-referencing what's actually stored with what's being used in dashboards and alerts. It exports the results as Prometheus metrics. This is intended as a self-hosted alternative to [Adaptive Metrics](https://grafana.com/docs/grafana-cloud/adaptive-telemetry/adaptive-metrics/) for Enterprise and open-source Mimir deployments.

On each analysis cycle (once per day by default), the tool:

1. **Discovers tenants** by querying the Mimir store-gateway for the list of active tenants.
2. **Analyzes dashboard usage** by running `mimirtool analyze grafana` against your Grafana instance to determine which metrics are referenced in dashboards.
3. Optionally analyzes alert usage by fetching provisioned alert rules from the Grafana API and checking which metrics appear in their expressions. This assumes the tenant ID is part of the datasource name to work, and is thus toggleable.
4. **Fetches top metrics by cardinality** for each tenant using Mimir's cardinality API (`/prometheus/api/v1/cardinality/label_values`), retrieving the top 100 metric names.
5. **Cross-references** the top metrics against dashboard and alert usage. Each metric is classified as either active or inactive and exported as a Prometheus gauge.

The output is a standard Prometheus gauge (`metric_active`) that you can visualize. The exported metric looks like:

```
metric_active{metric="some_metric_name", tenant="some-tenant"} 1
metric_active{metric="another_metric_name", tenant="some-tenant"} 0
```

A value of `1` means the metric is referenced in at least one dashboard or alert rule.

## Configuration

The tool is configured through a YAML file:

```yaml
grafana:
  url: "https://grafana.example.com"
  tokenFrom: "GRAFANA_TOKEN"   # read the token from this environment variable
  # token: "glsa_..."          # or specify it directly (not recommended)
  # insecure: false            # skip TLS verification (default: false)

mimir:
  querierUrl: "http://mimir-querier:8080"
  storeGatewayUrl: "http://mimir-store-gateway:8080"

http:
  host: "0.0.0.0"
  port: 8080
```

## CLI Usage

| Flag | Default | Description |
|---|---|---|
| `--config`, `-c` | (required) | Path to the YAML configuration file |
| `--output-dir`, `-o` | `.` | Directory for intermediate files produced by `mimirtool` |
| `--interval`, `-i` | `86400` | Seconds between analysis cycles (default is 24 hours) |
| `--disable-alert-correlation` | `false` | Skip alert rule analysis entirely |

For example, to run every 6 hours with alert correlation disabled:

```bash
cargo run -- --config config.yaml --interval 21600 --disable-alert-correlation
```

## Deploying to Kubernetes

A minimal installation looks like this:

```bash
helm install mimir-cardinality-analyzer oci://quay.io/duk4s/mimir-cardinality-analyzer-helm \
  --set grafana.url="https://grafana.example.com" \
  --set grafana.tokenFrom="GRAFANA_TOKEN" \
  --set mimir.querierUrl="http://mimir-querier:8080" \
  --set mimir.storeGatewayUrl="http://mimir-store-gateway:8080" \
  --set extraEnv[0].name="GRAFANA_TOKEN" \
  --set extraEnv[0].valueFrom.secretKeyRef.name="grafana-secret" \
  --set extraEnv[0].valueFrom.secretKeyRef.key="token"
```

See [values.yaml](./helm/mimir-cardinality-analyzer/values.yaml) for a full set of configurable values.

## Metrics

| Metric | Type | Labels | Description |
|---|---|---|---|
| `metric_active` | Gauge | `metric`, `tenant` | `1` if the metric is referenced in a dashboard or alert, `0` otherwise |
| `task_duration_seconds` | Histogram | `task`, `tenant_id` | Duration of internal analysis tasks (tenant fetching, dashboard analysis, etc.) |

## Limitations

- Only the top 100 metrics by cardinality are analyzed per tenant. Metrics outside that window are not evaluated.
- Alert rule matching relies on datasource names containing the tenant identifier, which assumes a naming convention in your Grafana datasource setup. If this doesn't match your setup, use `--disable-alert-correlation` to skip it.
