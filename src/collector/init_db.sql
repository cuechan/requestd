CREATE TABLE IF NOT EXISTS nodes (
	nodeid TEXT PRIMARY KEY,
	status TEXT NOT NULL,
	lastseen NUMERIC NOT NULL,
	firstseen NUMERIC NOT NULL,
	lastaddress TEXT NOT NULL,
	lastresponse TEXT NOT NULL DEFAULT '{}'
);
