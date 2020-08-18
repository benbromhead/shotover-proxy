#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Create a defered function to cleanup docker-compose. This also preserves the exit code before the trap was hit
# and returns that instead. So if our tests fails, our CI system will still pick it up!

function defer {
	docker-compose -f $SCRIPT_DIR/../../examples/redis-cluster/docker-compose.yml down
}

trap defer EXIT

docker-compose -f $SCRIPT_DIR/../../examples/redis-cluster/docker-compose.yml up -d
echo "Getting ready to run proxy"
sleep 5
echo "Running shotover"

cd $SCRIPT_DIR/../../
cargo flamegraph --bin shotover-proxy -- --config-file examples/redis-cluster/config.yaml




# ./src/redis-benchmark -t set,get,inc,lpush,rpush,lpop,rpop,sadd,hset,spop,lpush,lrange_100,lrange_600 -P 10