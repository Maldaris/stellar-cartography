#!/bin/bash
# Example API calls demonstrating rate limiting and request ID tracking

API_URL="http://localhost:3000"

echo "=== Testing Request ID Tracking ==="
echo "1. Request with custom request ID:"
curl -i -H "X-Request-Id: my-custom-id-123" \
     "$API_URL/health"

echo -e "\n\n2. Request without request ID (auto-generated):"
curl -i "$API_URL/health"

echo -e "\n\n=== Testing Rate Limiting ==="
echo "3. Making 15 rapid requests to test rate limiting:"

for i in {1..15}; do
    echo -e "\nRequest $i:"
    response=$(curl -s -w "\nStatus: %{http_code}" \
                    -H "X-Request-Id: test-rate-limit-$i" \
                    "$API_URL/systems/autocomplete?q=System")
    echo "$response"
    
    # Check if we got rate limited
    if [[ $response == *"429"* ]]; then
        echo ">>> Rate limited at request $i!"
    fi
done

echo -e "\n\n=== Testing Different Endpoints ==="
echo "4. Near systems query:"
curl -i -H "X-Request-Id: near-systems-test" \
     "$API_URL/systems/near?name=System_30000001&radius=1e15"

echo -e "\n\n5. Nearest systems query:"
curl -i -H "X-Request-Id: nearest-systems-test" \
     "$API_URL/systems/nearest?name=System_30000001&k=5"

echo -e "\n\n=== Testing Error Response with Request ID ==="
echo "6. Invalid system name:"
curl -i -H "X-Request-Id: error-test-123" \
     "$API_URL/systems/near?name=NonExistentSystem&radius=1e15"

echo -e "\n\nDone! Check the server logs to see request IDs in action." 