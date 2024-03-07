CREATE TABLE insulin_injections (
  user_id INTEGER NOT NULL,
  date_time DATETIME NOT NULL,
  cubic_centimeters FLOAT NOT NULL,
  PRIMARY KEY (user_id, date_time),
  FOREIGN KEY (user_id)
    REFERENCES users (id)
);
