server:
    cargo r -p blog-server --bin blog-server

registeruser:
    cargo r -p blog-cli --bin blog-cli -- --grpc register --username user --email user@domain2.dom --password 123

loginuser:
    cargo r -p blog-cli --bin blog-cli -- --grpc login --username user --password 123

logoutuser:
    cargo r -p blog-cli --bin blog-cli -- --grpc logout

list:
    cargo r -p blog-cli --bin blog-cli -- --grpc list

create1:
    cargo r -p blog-cli --bin blog-cli -- --grpc create --title title1 --content content1

get1:
    cargo r -p blog-cli --bin blog-cli -- --grpc get --id 1

delete1:
    cargo r -p blog-cli --bin blog-cli -- --grpc delete --id 1

showtoken:
    cat .blog_token