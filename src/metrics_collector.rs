use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use prometheus::{Encoder, IntCounter, Opts, Registry, TextEncoder, Gauge, Histogram, register_int_counter, IntCounterVec, IntGauge, register_int_counter_vec, register_int_gauge};
use warp::Filter;
use prometheus::{register_gauge, register_histogram};
use crate::config::Metrics;

#[derive(Clone)]
pub struct MetricsCollector {
    prometheus_handle: PrometheusHandle,
    metrics_config: Metrics,
    total_requests: IntCounter,
    memory_usage_gauge: Gauge,
    request_latency_histogram: Histogram,
    http_status_codes: IntCounterVec,
    backend_connections: IntGauge,
}

impl MetricsCollector {
    pub fn new(metrics_config: Metrics) -> MetricsCollector {
        let prometheus = PrometheusBuilder::new().build();
        let prometheus_handle = prometheus.handle();
        metrics::set_boxed_recorder(Box::new(prometheus)).unwrap();
        
        let total_requests = register_int_counter!("total_requests", "Total requests").unwrap();
        let memory_usage_gauge = register_gauge!("memory_usage", "Memory usage").unwrap();
        let request_latency_histogram = register_histogram!("request_latency", "Request latency").unwrap();
        let http_status_codes = register_int_counter_vec!("http_status_codes", "HTTP status codes", &["code"]).unwrap();
        let backend_connections = register_int_gauge!("backend_connections", "Backend connections").unwrap();
        // Register total_requests counter with Prometheus here

        MetricsCollector {
            prometheus_handle,
            metrics_config,
            total_requests,
            memory_usage_gauge,
            request_latency_histogram,
            http_status_codes,
            backend_connections,
        }
    }

    pub fn get_metrics_route(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("metrics").map(move || {
            let encoder = TextEncoder::new();
            let metric_families = prometheus::gather();
            let mut buffer = Vec::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();
            warp::reply::with_header(buffer, "content-type", encoder.format_type())
        })
    }

    pub async fn start_metrics_server(&self) {
        let metrics_server =
            warp::serve(self.get_metrics_route()).run(([0, 0, 0, 0], self.metrics_config.listen_port));
        metrics_server.await;
    }

    pub fn inc_total_requests(&self) {
        self.total_requests.inc();
    }

    pub fn set_memory_usage(&self, memory_usage: f64) {
        self.memory_usage_gauge.set(memory_usage);
    }

    pub fn observe_request_latency(&self, latency: f64) {
        self.request_latency_histogram.observe(latency);
    }

    pub fn inc_http_status_code(&self, code: u16) {
        self.http_status_codes.with_label_values(&[&code.to_string()]).inc();
    }

    pub fn set_backend_connections(&self, connections: i64) {
        self.backend_connections.set(connections);
    }

    pub fn get_prometheus_handle(&self) -> PrometheusHandle {
        self.prometheus_handle.clone()
    }

    pub fn get_total_requests(&self) -> IntCounter {
        self.total_requests.clone()
    }

    pub fn get_memory_usage_gauge(&self) -> Gauge {
        self.memory_usage_gauge.clone()
    }

    pub fn get_request_latency_histogram(&self) -> Histogram {
        self.request_latency_histogram.clone()
    }

    pub fn get_http_status_codes(&self) -> IntCounterVec {
        self.http_status_codes.clone()
    }

    pub fn get_backend_connections(&self) -> IntGauge {
        self.backend_connections.clone()
    }

    pub fn get_metrics_config(&self) -> Metrics {
        self.metrics_config.clone()
    }
}