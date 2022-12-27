curl -X 'POST' -H "Content-Type: application/json" -d '{"username": "testuser", "password": "grr"}'  http://localhost:8080/login/
curl -X 'GET'  http://localhost:8080/user/12
curl -X 'DELETE' http://localhost:8080/user/12
curl -X 'GET' http://localhost:8080/user
curl -X 'POST' -H "Content-Type: application/json" -d '{"username": "df", "password": "pass"}'  http://localhost:8080/login/
