import * as R from "ramda";

import getCortexDb from "@hypercortex/hypercortex-cli-client";

const hyper = async ({ modifications, taskAll, db }) => {
	const { set, auth } = R.pipe(
		R.filter(R.either(R.path(["prop", "set"]), R.path(["prop", "auth"]))),
		R.map(R.prop("prop")),
		R.mergeAll,
	)(modifications);

	if (set) {
		await getCortexDb(set);
		console.log(`changed hypercortex to "${set}"`);
		process.exit(0);
	}

	if (auth) {
		await new Promise(done => db.authorize(Buffer.from(auth, "hex"), done));
		console.log(`authorized "${auth}" to "${db.key.toString("hex")}"`);
	}
};

export default hyper;
