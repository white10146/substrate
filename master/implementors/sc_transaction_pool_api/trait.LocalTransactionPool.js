(function() {var implementors = {
"sc_transaction_pool":[["impl&lt;Block, Client&gt; LocalTransactionPool for <a class=\"struct\" href=\"sc_transaction_pool/struct.BasicPool.html\" title=\"struct sc_transaction_pool::BasicPool\">BasicPool</a>&lt;<a class=\"struct\" href=\"sc_transaction_pool/struct.FullChainApi.html\" title=\"struct sc_transaction_pool::FullChainApi\">FullChainApi</a>&lt;Client, Block&gt;, Block&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Block: <a class=\"trait\" href=\"sp_runtime/traits/trait.Block.html\" title=\"trait sp_runtime::traits::Block\">BlockT</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Client: <a class=\"trait\" href=\"sp_api/trait.ProvideRuntimeApi.html\" title=\"trait sp_api::ProvideRuntimeApi\">ProvideRuntimeApi</a>&lt;Block&gt; + BlockBackend&lt;Block&gt; + <a class=\"trait\" href=\"sp_blockchain/backend/trait.HeaderBackend.html\" title=\"trait sp_blockchain::backend::HeaderBackend\">HeaderBackend</a>&lt;Block&gt; + <a class=\"trait\" href=\"sp_runtime/traits/trait.BlockIdTo.html\" title=\"trait sp_runtime::traits::BlockIdTo\">BlockIdTo</a>&lt;Block&gt; + <a class=\"trait\" href=\"sp_blockchain/header_metadata/trait.HeaderMetadata.html\" title=\"trait sp_blockchain::header_metadata::HeaderMetadata\">HeaderMetadata</a>&lt;Block, Error = <a class=\"enum\" href=\"sp_blockchain/error/enum.Error.html\" title=\"enum sp_blockchain::error::Error\">Error</a>&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;Client: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;Client::<a class=\"associatedtype\" href=\"sp_api/trait.ProvideRuntimeApi.html#associatedtype.Api\" title=\"type sp_api::ProvideRuntimeApi::Api\">Api</a>: <a class=\"trait\" href=\"sp_transaction_pool/runtime_api/trait.TaggedTransactionQueue.html\" title=\"trait sp_transaction_pool::runtime_api::TaggedTransactionQueue\">TaggedTransactionQueue</a>&lt;Block&gt;,</span>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()