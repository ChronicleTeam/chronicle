set ADDR 'http://localhost:8000/api'

curl -X POST $ADDR/users \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "username=john@example.com&password=test123" \
    -c cookies.txt -b cookies.txt

curl -X POST $ADDR/login \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "username=test@example.com&password=test123" \
    -c cookies.txt -b cookies.txt

curl -b cookies.txt -X GET $ADDR/logout
