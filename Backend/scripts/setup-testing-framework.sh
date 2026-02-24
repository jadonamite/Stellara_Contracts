#!/bin/bash

# Setup script for Advanced Testing Framework
# This script installs all necessary dependencies and sets up the testing framework

set -e  # Exit immediately if a command exits with a non-zero status

echo "🚀 Setting up Advanced Testing Framework for Stellara Backend..."

# Navigate to backend directory
cd "$(dirname "$0")/.."

# Install all dependencies
echo "📦 Installing dependencies..."
npm install @pact-foundation/pact jest-pact fast-check @stryker-mutator/core @stryker-mutator/jest-runner nock puppeteer

# Verify installation
echo "🔍 Verifying installations..."
if command -v npx &> /dev/null; then
    echo "✅ npx is available"
else
    echo "❌ npx is not available"
    exit 1
fi

# Create necessary directories
echo "📁 Creating test directories..."
mkdir -p test/unit test/integration test/e2e test/contract test/property test/chaos test/mutation test/setup

# Create example test files
echo "📝 Creating example test files..."

# Contract test example
cat > test/examples/example.contract-spec.ts << 'EOF'
/**
 * Example Contract Test
 * Verifies API contracts between services
 */

describe('API Contract Tests', () => {
  it('should validate API response structure', () => {
    // Example contract test
    const response = { id: 1, name: 'test' };
    expect(response).toHaveProperty('id');
    expect(response).toHaveProperty('name');
  });
});
EOF

# Property-based test example
cat > test/examples/example.property-spec.ts << 'EOF'
/**
 * Example Property-Based Test
 * Uses fast-check for property-based testing
 */

import fc from 'fast-check';

describe('Property-Based Tests', () => {
  it('should validate string manipulation properties', () => {
    fc.assert(
      fc.property(fc.string(), (str) => {
        // Example property: reversing twice should return original
        const reversed = str.split('').reverse().join('');
        const doubleReversed = reversed.split('').reverse().join('');
        expect(doubleReversed).toBe(str);
      })
    );
  });
});
EOF

# Chaos test example
cat > test/examples/example.chaos-spec.ts << 'EOF'
/**
 * Example Chaos Engineering Test
 * Tests system resilience under failure conditions
 */

describe('Chaos Engineering Tests', () => {
  it('should handle simulated network delays', async () => {
    // Example chaos test - simulate network delay
    const startTime = Date.now();
    // Simulate delayed response
    await new Promise(resolve => setTimeout(resolve, 1000));
    const endTime = Date.now();
    
    // Verify system handles delay gracefully
    expect(endTime - startTime).toBeGreaterThanOrEqual(900);
  });
});
EOF

echo "✅ Advanced Testing Framework setup complete!"
echo ""
echo "📋 Next Steps:"
echo "1. Review TESTING_FRAMEWORK.md for detailed documentation"
echo "2. Customize test files in test/ directory for your specific needs"
echo "3. Run specific test types:"
echo "   - npm run test:contract    # Contract tests"
echo "   - npm run test:property    # Property-based tests"
echo "   - npm run test:chaos       # Chaos engineering tests"
echo "   - npm run test:integration # Integration tests"
echo "   - npm run test:mutation    # Mutation tests"
echo "   - npm run test:report      # Coverage report"
echo ""
echo "💡 Tip: Add your service-specific contract tests to validate API interactions"