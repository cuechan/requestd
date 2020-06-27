CREATE TABLE IF NOT EXISTS nodes (
	nodeid TEXT PRIMARY KEY,
	status TEXT NOT NULL,
	lastseen NUMERIC NOT NULL,
	firstseen NUMERIC NOT NULL,
	lastaddress TEXT NOT NULL,
	lastresponse TEXT NOT NULL DEFAULT '{}'
);


CREATE TABLE IF NOT EXISTS trigger (
	nodeid TEXT PRIMARY KEY,
	status TEXT,
	UNIQUE(nodeid, status)
);


CREATE TABLE IF NOT EXISTS responses (
	nodeid TEXT,
	timestamp TEXT,
	category TEXT,
	data TEXT,
	UNIQUE(nodeid, timestamp)
);
