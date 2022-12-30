HOST=127.0.0.1:8080

echo "login..."
curl -X POST  "${HOST}/login" -c "/tmp/cookie" \
--header 'Content-Type: application/json' \
--data-raw '{
    "username": "testuser",
    "password": "grr"
}'

echo "hello..."
curl -b "/tmp/cookie" -X GET ${HOST}/hello

echo "users..."
curl -b "/tmp/cookie" -X GET ${HOST}/x/
#curl -X 'DELETE' http://localhost:8080/user/12
#curl -X 'GET' http://localhost:8080/user
#curl -X 'POST' -H "Content-Type: application/json" -d '{"username": "df", "password": "pass"}'  http://localhost:8080/login/
