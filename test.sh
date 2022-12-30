HOST=127.0.0.1:8080

echo "login..."
curl -X POST  "${HOST}/login" -c "/tmp/cookie" \
--header 'Content-Type: application/json' \
--data-raw '{
    "username": "testuser",
    "password": "grr"
}'

echo "hello..."
curl -b "/tmp/cookie" ${HOST}/hello

echo "users..."
curl -b "/tmp/cookie" ${HOST}/user

echo "add user..."
curl -b "/tmp/cookie" -X 'POST' -H "Content-Type: application/json" -d '{"username": "df", "password": "pass"}' ${HOST}/user

echo "users..."
curl -b "/tmp/cookie" ${HOST}/user

echo "delete user..."
curl -b "/tmp/cookie" -X 'DELETE' ${HOST}/user/0
