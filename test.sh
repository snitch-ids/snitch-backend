curl -X 'POST' -H "Content-Type: application/json" -d '{"id": 12, "name": "df"}'  http://localhost:8080/user/
curl -X 'GET'  http://localhost:8080/user/12
curl -X 'DELETE' http://localhost:8080/user/12
curl -X 'GET' http://localhost:8080/user
curl -X 'POST' -H "Content-Type: application/json" -d '{"id": 12, "name": "df"}'  http://localhost:8080/user/
