CREATE TABLE sugar_level_measurements (
  user_id INTEGER NOT NULL,
  date_time DATETIME NOT NULL,
  millimoles_per_liter FLOAT NOT NULL,
  PRIMARY KEY (user_id, date_time),
  FOREIGN KEY (user_id)
    REFERENCES users (id)
);
