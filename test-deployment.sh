#!/bin/bash

echo "🚀 Testing TRX Tracker Unified Deployment"
echo "========================================"

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed"
    exit 1
fi

# Check if Docker Compose is available
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed"
    exit 1
fi

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cp .env.example .env
fi

# Build and start services
echo "🔨 Building and starting services..."
docker-compose up -d --build

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 30

# Test health endpoint
echo "🏥 Testing health endpoint..."
if curl -f http://localhost:8080/health > /dev/null 2>&1; then
    echo "✅ Health check passed"
else
    echo "❌ Health check failed"
    docker-compose logs tron-tracker
    exit 1
fi

# Test admin UI
echo "🎛️ Testing admin UI..."
if curl -f http://localhost:3000 > /dev/null 2>&1; then
    echo "✅ Admin UI is accessible"
else
    echo "❌ Admin UI is not accessible"
    exit 1
fi

# Test API endpoints
echo "📡 Testing API endpoints..."
if curl -f http://localhost:8080/api/v1/dashboard/stats > /dev/null 2>&1; then
    echo "✅ API endpoints are working"
else
    echo "❌ API endpoints are not working"
    exit 1
fi

echo ""
echo "🎉 All tests passed!"
echo "📊 Admin UI: http://localhost:3000"
echo "🔌 API: http://localhost:8080"
echo "📚 Health: http://localhost:8080/health"
echo ""
echo "To stop the services: docker-compose down"
