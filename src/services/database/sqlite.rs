use rusqlite::Connection;

pub fn migrate() {
    let connection = Connection::open("./database/rusted.db").unwrap();

    let create_commands_table_query = "
      CREATE TABLE IF NOT EXISTS commands (
        id INTEGER PRIMARY KEY,
        name TEXT UNIQUE NOT NULL,
        response TEXT NOT NULL,
        created_at DATETIME NOT NULL,
        updated_at DATETIME NOT NULL,
        deleted_at DATETIME
      )
    ";

    connection.execute(create_commands_table_query, []).unwrap();
}

pub fn create_command(name: &str, response: &str) {
    let connection = Connection::open("./database/rusted.db").unwrap();

    let create_command_query = "
      INSERT INTO commands (name, response, created_at, updated_at)
      VALUES (?1, ?2, datetime('now'), datetime('now'))
    ";

    connection
        .execute(create_command_query, &[&name, &response])
        .unwrap();
}

pub fn get_command_response(name: &str) -> Result<String, rusqlite::Error> {
    let connection = Connection::open("./database/rusted.db").unwrap();

    let get_command_response_query = "
      SELECT response
      FROM commands
      WHERE name = ?
      AND deleted_at IS NULL
    ";

    let mut statement = connection.prepare(get_command_response_query).unwrap();
    let responses = statement.query_map(&[&name], |row| Ok(row.get(0).unwrap()));

    match responses {
        Ok(responses) => {
            for response in responses {
                return Ok(response.unwrap());
            }
        }
        Err(e) => return Err(e),
    }

    Err(rusqlite::Error::QueryReturnedNoRows)
}

pub fn update_command_response(name: &str, response: &str) {
    let connection = Connection::open("./database/rusted.db").unwrap();

    let update_command_response_query = "
      UPDATE commands
      SET response = ?, updated_at = datetime('now')
      WHERE name = ?
    ";

    connection
        .execute(update_command_response_query, &[&response, &name])
        .unwrap();
}

pub fn delete_command(name: &str) {
    let connection = Connection::open("./database/rusted.db").unwrap();

    let delete_command_query = "
      UPDATE commands
      SET deleted_at = datetime('now')
      WHERE name = ?
    ";

    connection.execute(delete_command_query, &[&name]).unwrap();
}
