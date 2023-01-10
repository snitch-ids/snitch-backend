set -e

HOST=http://127.0.0.1:8081

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

echo "generate token... (cropping quotes form JSON)"
string=$(curl -b "/tmp/cookie" ${HOST}/user/1/token/new)
string2=${string#'"'}
string2=${string2%'"'}

curl -b "/tmp/cookie" ${HOST}/user/1/token/

echo "testing a token: ${string2}"
curl -X POST -H "content-type:application/json" -H "authorization: Bearer ${string2}" ${HOST}/messages/ --data-raw '{
    "hostname": "apple",
    "title": "title",
    "content": "content",
    "timestamp": "2022-01-02T12:12:12Z"
}'

ADMIN_TOKEN="!!!INSECUREADMINTOKEN!!!"
echo "test admin token"
curl -X POST -H "content-type:application/json" -H "authorization: Bearer ${ADMIN_TOKEN}" ${HOST}/messages/ --data-raw '{
    "hostname": "admin",
    "title": "admintitle",
    "content": "content",
    "timestamp": "2022-01-02T12:12:12Z"
}'

echo "test getting messages"
curl -v -b "/tmp/cookie" -X POST -H "content-type:application/json" ${HOST}/messages/all/ --data-raw '{
    "hostname": "admin"
}'

echo "test getting messages without token. this should throw 401 unauth"
curl -X POST -H "content-type:application/json" ${HOST}/messages/all/ --data-raw '{
    "hostname": "admin"
}'

echo "done."



# echo "testing an INVALID token"
# curl  -X POST  -H "Content-Type:application/json" -H "Authorization: Bearer INVEALIDTOKEN123" 127.0.0.1:8080/messages/ --data-raw '{
#     "hostname": "apple",
#     "title": "title",
#     "content": "content",
#     "timestamp": "2022-01-02T12:12:12Z"
# }'
# 
