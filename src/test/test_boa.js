
class AvocadoPlugin {
    constructor(e) {
        this.e = e
    }
    async check () {
        logger.info(this.e.msg)
        return check(/^#测试/, this.e.msg);
    }

    async process () {
        console.log(this.e.msg)
        // return this.e.msg
        return avocado(this.e.msg)
    }
}




async function wrapper(p) {
    console.log("into wrapper")
    let match = await p.check();
    console.log({match})
    if (match) {
        let res = await p.process();
        console.log(res)
        return res
    }
    return "not match";
}

let emulated = {
    msg: '#测试测试'
};
let plugin = new AvocadoPlugin(emulated);
console.log('emulated:', emulated)

new Promise((resolve, reject) => {
    wrapper(plugin).then(res => {
        resolve(res)
    }).catch(err => {
        console.error(err)
        reject(err)
    })
})



