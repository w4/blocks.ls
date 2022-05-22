import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime.js";
import localizedFormat from "dayjs/plugin/localizedFormat.js";

import "dayjs/locale/en-gb";

dayjs.locale("en-gb");

dayjs.extend(relativeTime);
dayjs.extend(localizedFormat);

export { dayjs };
