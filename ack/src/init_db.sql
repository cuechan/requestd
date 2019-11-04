CREATE TABLE IF NOT EXISTS nodes (
	nodeid TEXT,
	nodeinfo TEXT,
	last_update TEXT,
	statistics TEXT
);

CREATE TABLE IF NOT EXISTS raw_responses (
	timestamp INTEGER,
	remote TEXT,
	response TEXT
);
