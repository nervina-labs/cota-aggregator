const { addressToScript, serializeScript } = require('@nervosnetwork/ckb-sdk-utils')
const { Collector, Aggregator, generateTransferCotaTx } = require("@nervina-labs/cota-sdk");

const RECEIVER2_PRIVATE_KEY = '0x02365419c63e3f2e5ca75f2be80a38e3da4eb9a844b4963bc2444ab72b38c6d2'
const RECEIVER2_ADDRESS = 'ckt1qyqz8vxeyrv4nur4j27ktp34fmwnua9wuyqqggd748'
const ISSUER_ADDRESS = 'ckt1qyq0scej4vn0uka238m63azcel7cmcme7f2sxj5ska'
const RECEIVER1_ADDRESS = 'ckt1qyqrq7vdeh5a8rnp4n2tuuu08p5uw8a5qdtqrpvdsg'

const secp256k1CellDep = async (ckb) => {
  const secp256k1Dep = (await ckb.loadDeps()).secp256k1Dep
  return { outPoint: secp256k1Dep.outPoint, depType: 'depGroup' }
}

const run = async () => {
  const service = {
    collector: new Collector({ ckbNodeUrl: 'http://localhost:8114', ckbIndexerUrl: 'http://localhost:8116' }),
    aggregator: new Aggregator({ registryUrl: 'http://localhost:3050', cotaUrl: 'http://localhost:3030' }),
  }
  const ckb = service.collector.getCkb()
  const cotaLock = addressToScript(RECEIVER2_ADDRESS)
  const withdrawLock = addressToScript(ISSUER_ADDRESS)

  const transfers = [
    {
      cotaId: '0x096b5d210b3b32fab6f8fbd937e21b06b5d91e86',
      tokenIndex: "0x00000015",
      toLockScript: serializeScript(addressToScript(RECEIVER1_ADDRESS)),
    },
  ]
  let rawTx = await generateTransferCotaTx(service, cotaLock, withdrawLock, transfers)

  const secp256k1Dep = await secp256k1CellDep(ckb)
  rawTx.cellDeps.push(secp256k1Dep)

  const signedTx = ckb.signTransaction(RECEIVER2_PRIVATE_KEY)(rawTx)
  console.log(JSON.stringify(signedTx))
  let txHash = await ckb.rpc.sendTransaction(signedTx, 'passthrough')
  console.info(`Transfer cota nft tx has been sent with tx hash ${txHash}`)
}


 run()

