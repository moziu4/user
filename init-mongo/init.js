db = db.getSiblingDB("mydb");
db.createUser({
    user: "dev",
    pwd: "dev_pass",
    roles: [
        { role: "readWrite", db: "mydb" }
    ]
});

db.createCollection("data");