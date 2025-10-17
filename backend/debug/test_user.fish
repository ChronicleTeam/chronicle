set ADDR 'http://localhost:8000/api'

curl -b cookies.txt -X POST $ADDR/users \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "username=john@example.com&password=test123"

curl -b cookies.txt -X POST $ADDR/login \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "username=test@example.com&password=test123" \
    -c cookies.txt

curl -b cookies.txt -X GET $ADDR/logout


curl -b cookies.txt -X PATCH $ADDR/users/1 \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "username=ch_admin@example.com"

curl -i -X POST $ADDR/login \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "username=test@example.com&password=test123" \
    | grep -i "set-cookie"