// mongosh --authenticationDatabase admin -u "dev" -p "dev_pass"
db = db.getSiblingDB("user");
db.createUser({
    user: "dev",
    pwd: "dev_pass",
    roles: [
        { role: "readWrite", db: "user" }
    ]
});

db.createCollection("users");
db.createCollection("relationship");
db.createCollection("auth");
db.createCollection("memberships");
db.createCollection("phones");
db.createCollection("addresses");