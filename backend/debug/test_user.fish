set ADDR 'http://localhost:8000/api'

curl -X POST $ADDR/register \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "username=test@example.com&password=test123" \
    -c cookies.txt -b cookies.txt

curl -X POST $ADDR/login \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "username=test@example.com&password=test123" \
    -c cookies.txt -b cookies.txt

curl -X GET $ADDR/logout \
    -b cookies.txt