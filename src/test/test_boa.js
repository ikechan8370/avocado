
class AvocadoPlugin {
    async matches (e) {
        return e.msg.test(/^#测试/);
    }

    async process (e) {
        console.log(e.msg)
        return test(e.msg)
    }
}

let plugin = new AvocadoPlugin();

function wrapper(p, e) {
    return p.matches(e).then((result) => {
        if (result) {
            p.process(e).then((result) => {
                console.log(result)
                return result
            })
        }
    })
}

let emulated = {
    msg: '#测试测试'
};
console.log('emulated:', emulated)
wrapper(plugin, emulated)
