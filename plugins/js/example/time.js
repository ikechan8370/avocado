new Promise(async (resolve, reject) => {
    let match = check(/^!时间/, e.msg);
    if (match) {
        logger.info(`[time] (external JS plugin) ${e.msg}`)
        let str = new Date().toString();
        await e.reply(str)
    }
    resolve()
})

