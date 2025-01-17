use criterion::{criterion_group, criterion_main, Criterion};
use redis::Cmd;
use std::path::Path;
use test_helpers::docker_compose::DockerCompose;
use test_helpers::lazy::new_lazy_shared;

#[path = "../tests/helpers/mod.rs"]
mod helpers;
use helpers::ShotoverManager;

struct Query {
    name: &'static str,
    command: Cmd,
}

fn redis(c: &mut Criterion) {
    let mut group = c.benchmark_group("redis");
    group.throughput(criterion::Throughput::Elements(1));
    group.noise_threshold(0.2);

    let queries = [
        Query {
            name: "set",
            command: redis::cmd("SET").arg("foo").arg(42).clone(),
        },
        Query {
            name: "get",
            command: redis::cmd("GET").arg("bench_test_data").clone(),
        },
    ];

    {
        let resources = new_lazy_shared(|| {
            BenchResources::new(
                "example-configs/redis-multi/topology.yaml",
                "example-configs/redis-multi/docker-compose.yml",
            )
        });
        for query in &queries {
            group.bench_with_input(
                format!("active_{}", query.name),
                &resources,
                move |b, resources| {
                    b.iter(|| {
                        let mut resources = resources.borrow_mut();
                        query
                            .command
                            .execute(&mut resources.as_mut().unwrap().connection);
                    })
                },
            );
        }
    }

    {
        let resources = new_lazy_shared(|| {
            BenchResources::new(
                "example-configs/redis-cluster/topology.yaml",
                "example-configs/redis-cluster/docker-compose.yml",
            )
        });
        for query in &queries {
            group.bench_with_input(
                format!("cluster_{}", query.name),
                &resources,
                move |b, resources| {
                    b.iter(|| {
                        let mut resources = resources.borrow_mut();
                        query
                            .command
                            .execute(&mut resources.as_mut().unwrap().connection);
                    })
                },
            );
        }
    }

    {
        let resources = new_lazy_shared(|| {
            BenchResources::new(
                "example-configs/redis-passthrough/topology.yaml",
                "example-configs/redis-passthrough/docker-compose.yml",
            )
        });
        for query in &queries {
            group.bench_with_input(
                format!("passthrough_{}", query.name),
                &resources,
                move |b, resources| {
                    b.iter(|| {
                        let mut resources = resources.borrow_mut();
                        query
                            .command
                            .execute(&mut resources.as_mut().unwrap().connection);
                    })
                },
            );
        }
    }

    // Only need to test one case here as the type of command shouldnt affect TLS
    group.bench_with_input(
        "single_tls",
        || {
            test_helpers::cert::generate_redis_test_certs(Path::new(
                "example-configs/redis-tls/certs",
            ));
            BenchResources::new(
                "example-configs/redis-tls/topology.yaml",
                "example-configs/redis-tls/docker-compose.yml",
            )
        },
        move |b, state| {
            b.iter(|| {
                redis::cmd("SET")
                    .arg("foo")
                    .arg(42)
                    .execute(&mut state.connection);
            })
        },
    );

    // Only need to test one case here as the type of command shouldnt affect TLS
    group.bench_with_input(
        "cluster_tls",
        || {
            test_helpers::cert::generate_redis_test_certs(Path::new(
                "example-configs/redis-tls/certs",
            ));
            BenchResources::new(
                "example-configs/redis-cluster-tls/topology.yaml",
                "example-configs/redis-cluster-tls/docker-compose.yml",
            )
        },
        move |b, state| {
            b.iter(|| {
                redis::cmd("SET")
                    .arg("foo")
                    .arg(42)
                    .execute(&mut state.connection);
            })
        },
    );
}

criterion_group!(benches, redis);
criterion_main!(benches);

struct BenchResources {
    _compose: DockerCompose,
    _shotover_manager: ShotoverManager,
    connection: redis::Connection,
}

impl BenchResources {
    fn new(shotover_topology: &str, compose_file: &str) -> Self {
        let compose = DockerCompose::new(compose_file);
        let shotover_manager = ShotoverManager::from_topology_file(shotover_topology);

        let mut connection = shotover_manager.redis_connection(6379);
        redis::cmd("SET")
            .arg("bench_test_data")
            .arg("A value with some length length to it to form a reasonable benchmark")
            .execute(&mut connection);

        Self {
            _compose: compose,
            _shotover_manager: shotover_manager,
            connection,
        }
    }
}
