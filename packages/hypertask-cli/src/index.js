import * as R from "ramda";
import os from "os";

import getCortexDb from "@hypercortex/hypercortex-cli-client";
import createTask from "@hypercortex/object-type-task";
import createTelemetry from "@hypercortex/object-type-telemetry";

import partitionCommandsAndArgs from "./parseArgs";

import add from "./commands/add";
import hyper from "./commands/hyper";
import basicDisplay from "./commands/basicDisplay";
import done from "./commands/done";
import modify from "./commands/modify";
import snooze from "./commands/snooze";

const commandToFunction = { add, hyper, done, modify, snooze };

const main = async () => {
	const db = await getCortexDb();

	console.log(
		`cortex: "${db.key.toString("hex")}"
local:  "${db.local.key.toString("hex")}"`,
	);

	const { task, taskAll } = createTask(db);
	const { telemetry } = createTelemetry(db);

	telemetry(db.local.key.toString("hex")).nameSet(os.hostname());

	const { filter, command, modifications } = partitionCommandsAndArgs(
		commandToFunction,
	)(process.argv);

	await (commandToFunction[command] || basicDisplay)({
		filter,
		modifications,
		taskAll,
		task,
		db,
	});
};

try {
	main();
} catch (e) {
	console.error(e);
}
