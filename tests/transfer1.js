const { addressToScript, serializeScript } = require('@nervosnetwork/ckb-sdk-utils')
const { Collector, Aggregator, generateTransferCotaTx } = require("@nervina-labs/cota-sdk");

const RECEIVER1_PRIVATE_KEY = '0xaa3bafdf2dc710aa408d81d8f577d820e77be682bdf870e0ceef6a7fdbd7cea4'
const RECEIVER1_ADDRESS = 'ckt1qyqrq7vdeh5a8rnp4n2tuuu08p5uw8a5qdtqrpvdsg'
const ISSUER_ADDRESS = 'ckt1qyq0scej4vn0uka238m63azcel7cmcme7f2sxj5ska'
const RECEIVER2_ADDRESS = 'ckt1qyqz8vxeyrv4nur4j27ktp34fmwnua9wuyqqggd748'

const secp256k1CellDep = () => {
  return { outPoint: {
      txHash: "0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37",
      index: "0x0",
    }, depType: 'depGroup' }
}

const run = async () => {
  const service = {
    collector: new Collector({ ckbNodeUrl: 'http://localhost:8114', ckbIndexerUrl: 'http://localhost:8116' }),
    aggregator: new Aggregator({ registryUrl: 'http://localhost:3050', cotaUrl: 'http://localhost:3030' }),
  }
  const ckb = service.collector.getCkb()
  const cotaLock = addressToScript(RECEIVER1_ADDRESS)
  const withdrawLock = addressToScript(ISSUER_ADDRESS)

  const transfers = [
    {
      cotaId: '0x096b5d210b3b32fab6f8fbd937e21b06b5d91e86',
      tokenIndex: "0x0000001a",
      toLockScript: serializeScript(addressToScript(RECEIVER2_ADDRESS)),
    },
  ]
  let rawTx = await generateTransferCotaTx(service, cotaLock, withdrawLock, transfers)

  const secp256k1Dep = await secp256k1CellDep()
  rawTx.cellDeps.push(secp256k1Dep)

  const signedTx = ckb.signTransaction(RECEIVER1_PRIVATE_KEY)(rawTx)
  console.log(JSON.stringify(signedTx))
  let txHash = await ckb.rpc.sendTransaction(signedTx, 'passthrough')
  console.info(`Transfer cota nft tx has been sent with tx hash ${txHash}`)
}


 run()

