use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ecl_runtime::ECLRuntime;

fn create_test_yaml() {
    let yaml = r#"
server:
  port: 8080
  host: localhost
  debug: true
  max_connections: 1000
  request_timeout: 30
  ssl_enabled: true
database:
  url: postgresql://localhost
  pool_size: 10
  max_overflow: 20
  connection_timeout: 5
  idle_timeout: 300
cache:
  enabled: true
  ttl: 3600
  backend: redis
  max_size: 1000
logging:
  level: debug
  format: json
  outputs:
    - stdout
    - file
  file_path: /var/log/app.log
"#;
    std::fs::write("bench.yaml", yaml).ok();
}

fn create_test_json() {
    let json = r#"
{
  "server": {
    "port": 8080,
    "host": "localhost",
    "debug": true,
    "max_connections": 1000
  },
  "database": {
    "url": "postgresql://localhost",
    "pool_size": 10
  },
  "cache": {
    "enabled": true,
    "ttl": 3600
  }
}
"#;
    std::fs::write("bench.json", json).ok();
}

fn benchmark_yaml(c: &mut Criterion) {
    create_test_yaml();
    c.bench_function("ecl_load_yaml", |b| {
        b.iter(|| ECLRuntime::load_yaml(black_box("bench.yaml")))
    });
}

fn benchmark_json(c: &mut Criterion) {
    create_test_json();
    c.bench_function("ecl_load_json", |b| {
        b.iter(|| ECLRuntime::load_json(black_box("bench.json")))
    });
}

criterion_group!(benches, benchmark_yaml, benchmark_json);
criterion_main!(benches);
