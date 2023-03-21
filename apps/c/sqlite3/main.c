#include "sqlite3.h"
#include <stdio.h>
#include <stddef.h>

int callback(void *NotUsed, int argc, char **argv, char **azColName)
{
    NotUsed = NULL;

    for (int i = 0; i < argc; ++i)
    {
        printf("%s = %s\n", azColName[i], (argv[i] ? argv[i] : "NULL"));
    }

    printf("\n");

    return 0;
}

void exec(sqlite3 *db, char *sql) {
    printf("\nsqlite exec\n%s\n", sql);
    char * errmsg = NULL;
    int rc = sqlite3_exec(db, sql, NULL, NULL, &errmsg);

    if(rc != SQLITE_OK)
    {
        printf("%s\n",errmsg);
    }
}

void query(sqlite3 *db, char *sql) {
    printf("\nsqlite query\n%s\n", sql);
    char * errmsg = NULL;
    int rc = sqlite3_exec(db, sql, callback, NULL, &errmsg);

    if(rc != SQLITE_OK)
    {
        printf("%s\n",errmsg);
    }
}


int main() {
    printf("sqlite version%s\n", sqlite3_libversion());
    sqlite3 *db;
    int ret = sqlite3_open(":memory:", &db);
    printf("sqlite open memory status %d \n", ret);

    printf("init user table\n");
    exec(db, "create table user("
        "id INTEGER PRIMARY KEY AUTOINCREMENT,"
        "username TEXT,"
        "password TEXT"
    ")");

    printf("insert user 1、2、3 into user table");

    exec(db, "insert into user (username, password) VALUES ('1', 'password1'), ('2', 'password2'), ('3', 'password3')");

    printf("select all");
    query(db, "select * from user");

    printf("select id = 2");
    query(db, "select * from user where id = 2");

    return 0;
}