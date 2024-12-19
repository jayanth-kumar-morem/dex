#!/bin/bash

# Check API health
check_api() {
    if curl -s -f http://localhost:8080/health > /dev/null; then
        echo "API: ✅ Healthy"
    else
        echo "API: ❌ Unhealthy"
        return 1
    fi
}

# Check Redis connection
check_redis() {
    if redis-cli ping > /dev/null 2>&1; then
        echo "Redis: ✅ Healthy"
    else
        echo "Redis: ❌ Unhealthy"
        return 1
    fi
}

# Check PostgreSQL connection
check_postgres() {
    if pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
        echo "PostgreSQL: ✅ Healthy"
    else
        echo "PostgreSQL: ❌ Unhealthy"
        return 1
    fi
}

# Check Core Service
check_core() {
    if curl -s -f http://localhost:9090/health > /dev/null; then
        echo "Core Service: ✅ Healthy"
    else
        echo "Core Service: ❌ Unhealthy"
        return 1
    fi
}

# Run all checks
echo "Running health checks..."
echo "======================="

check_api
check_redis
check_postgres
check_core

# Check if any service failed
if [ $? -eq 0 ]; then
    echo "======================="
    echo "All services are healthy! 🚀"
    exit 0
else
    echo "======================="
    echo "Some services are unhealthy! ⚠️"
    exit 1
fi 