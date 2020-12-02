// This file is part of Substrate.

// Copyright (C) 2018-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub use frame_metadata::{
	DecodeDifferent, FnEncode, RuntimeMetadata, ModuleMetadata, RuntimeMetadataLastVersion,
	DefaultByteGetter, RuntimeMetadataPrefixed, StorageEntryMetadata, StorageMetadata,
	StorageEntryType, StorageEntryModifier, DefaultByte, StorageHasher, ModuleErrorMetadata,
	ExtrinsicMetadata,
};

pub use frame_metadata::vnext;

/// Implements the metadata support for the given runtime and all its modules.
///
/// Example:
/// ```
///# mod module0 {
///#    // Kind of alias for `Config` trait. Deprecated as `Trait` is renamed `Config`.
///#    pub trait Trait: Config {}
///#    impl<T: Config> Trait for T {}
///#    pub trait Config: 'static {
///#        type Origin;
///#        type BlockNumber;
///#        type PalletInfo: frame_support::traits::PalletInfo;
///#        type DbWeight: frame_support::traits::Get<frame_support::weights::RuntimeDbWeight>;
///#    }
///#    frame_support::decl_module! {
///#        pub struct Module<T: Trait> for enum Call where origin: T::Origin, system=self {}
///#    }
///#
///#    frame_support::decl_storage! {
///#        trait Store for Module<T: Trait> as TestStorage {}
///#    }
///# }
///# use module0 as module1;
///# use module0 as module2;
///# impl module0::Config for Runtime {
///#     type Origin = u32;
///#     type BlockNumber = u32;
///#     type PalletInfo = ();
///#     type DbWeight = ();
///# }
///#
///# type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<(), (), (), ()>;
///
/// struct Runtime;
/// frame_support::impl_runtime_metadata! {
///     for Runtime with modules where Extrinsic = UncheckedExtrinsic
///         module0::Module as Module0 { index 0 } with,
///         module1::Module as Module1 { index 1 } with,
///         module2::Module as Module2 { index 2 } with Storage,
/// };
/// ```
///
/// In this example, just `MODULE3` implements the `Storage` trait.
#[macro_export]
macro_rules! impl_runtime_metadata {
	(
		for $runtime:ident with modules where Extrinsic = $ext:ident
			$( $rest:tt )*
	) => {
		impl $runtime {
			pub fn metadata() -> $crate::metadata::vnext::RuntimeMetadataPrefixed<scale_info::form::CompactForm> {
				let mut registry = ::scale_info::Registry::new();
				let metadata =
					$crate::metadata::vnext::RuntimeMetadataLastVersion {
							modules: $crate::__runtime_modules_to_metadata!($runtime;; $( $rest )*),
							// extrinsic: $crate::metadata::ExtrinsicMetadata {
							// 	version: <$ext as $crate::sp_runtime::traits::ExtrinsicMetadata>::VERSION,
							// 	signed_extensions: <
							// 			<
							// 				$ext as $crate::sp_runtime::traits::ExtrinsicMetadata
							// 			>::SignedExtensions as $crate::sp_runtime::traits::SignedExtension
							// 		>::identifier()
							// 			.into_iter()
							// 			.map($crate::metadata::DecodeDifferent::Encode)
							// 			.collect(),
							// },
				};
				use ::scale_info::IntoCompact as _;
				metadata.into_compact(&mut registry).into()
			}
		}
	}
}

#[macro_export]
#[doc(hidden)]
macro_rules! __runtime_modules_to_metadata {
	(
		$runtime: ident;
		$( $metadata:expr ),*;
		$mod:ident::$module:ident $( < $instance:ident > )? as $name:ident
			{ index $index:tt }
			$(with)+ $($kw:ident)*
		,
		$( $rest:tt )*
	) => {
		$crate::__runtime_modules_to_metadata!(
			$runtime;
			$( $metadata, )* $crate::metadata::vnext::ModuleMetadata {
				name: stringify!($name),
				index: $index,
				// storage: $crate::__runtime_modules_to_metadata_calls_storage!(
				// 	$mod, $module $( <$instance> )?, $runtime, $(with $kw)*
				// ),
				calls: $crate::__runtime_modules_to_metadata_calls_call!(
					$mod, $module $( <$instance> )?, $runtime, $(with $kw)*
				),
				event: $crate::__runtime_modules_to_metadata_calls_event!(
					$mod, $module $( <$instance> )?, $runtime, $(with $kw)*
				),
				// constants: $crate::metadata::DecodeDifferent::Encode(
				// 	$crate::metadata::FnEncode(
				// 		$mod::$module::<$runtime $(, $mod::$instance )?>::module_constants_metadata
				// 	)
				// ),
				// errors: $crate::metadata::DecodeDifferent::Encode(
				// 	$crate::metadata::FnEncode(
				// 		<$mod::$module::<$runtime $(, $mod::$instance )?> as $crate::metadata::ModuleErrorMetadata>::metadata
				// 	)
				// )
			};
			$( $rest )*
		)
	};
	(
		$runtime:ident;
		$( $metadata:expr ),*;
	) => {
		vec![$( $metadata ),* ]
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __runtime_modules_to_metadata_calls_call {
	(
		$mod: ident,
		$module: ident $( <$instance:ident> )?,
		$runtime: ident,
		with Call
		$(with $kws:ident)*
	) => {
		Some($mod::$module::<$runtime $(, $mod::$instance )?>::call_functions())
	};
	(
		$mod: ident,
		$module: ident $( <$instance:ident> )?,
		$runtime: ident,
		with $_:ident
		$(with $kws:ident)*
	) => {
		$crate::__runtime_modules_to_metadata_calls_call! {
			$mod, $module $( <$instance> )?, $runtime, $(with $kws)*
		};
	};
	(
		$mod: ident,
		$module: ident $( <$instance:ident> )?,
		$runtime: ident,
	) => {
		None
	};
}


#[macro_export]
#[doc(hidden)]
macro_rules! __runtime_modules_to_metadata_calls_event {
	(
		$mod: ident,
		$module: ident $( <$instance:ident> )?,
		$runtime: ident,
		with Event
		$(with $kws:ident)*
	) => {
		Some($crate::paste::expr!{
				$runtime:: [< __module_events_ $mod $(_ $instance)?>]()
			}
		)
	};
	(
		$mod: ident,
		$module: ident $( <$instance:ident> )?,
		$runtime: ident,
		with $_:ident
		$(with $kws:ident)*
	) => {
		$crate::__runtime_modules_to_metadata_calls_event!( $mod, $module $( <$instance> )?, $runtime, $(with $kws)* );
	};
	(
		$mod: ident,
		$module: ident $( <$instance:ident> )?,
		$runtime: ident,
	) => {
		None
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __runtime_modules_to_metadata_calls_storage {
	(
		$mod: ident,
		$module: ident $( <$instance:ident> )?,
		$runtime: ident,
		with Storage
		$(with $kws:ident)*
	) => {
		Some($crate::metadata::DecodeDifferent::Encode(
			$crate::metadata::FnEncode(
				$mod::$module::<$runtime $(, $mod::$instance )?>::storage_metadata
			)
		))
	};
	(
		$mod: ident,
		$module: ident $( <$instance:ident> )?,
		$runtime: ident,
		with $_:ident
		$(with $kws:ident)*
	) => {
		$crate::__runtime_modules_to_metadata_calls_storage! {
			$mod, $module $( <$instance> )?, $runtime, $(with $kws)*
		};
	};
	(
		$mod: ident,
		$module: ident $( <$instance:ident> )?,
		$runtime: ident,
	) => {
		None
	};
}


#[cfg(test)]
// Do not complain about unused `dispatch` and `dispatch_aux`.
#[allow(dead_code)]
mod tests {
	use super::*;
	use frame_metadata::DefaultByte;
	use codec::{Encode, Decode};
	use crate::traits::Get;
	use scale_info::{Registry, IntoCompact};
	use sp_runtime::{
		traits::Member,
		transaction_validity::TransactionValidityError
	};
	use sp_std::marker::PhantomData;
	use scale_info::form::CompactForm;

	#[derive(Clone, Eq, Debug, PartialEq, Encode, Decode)]
	struct TestExtension;
	impl sp_runtime::traits::SignedExtension for TestExtension {
		type AccountId = u32;
		type Call = ();
		type AdditionalSigned = u32;
		type Pre = ();
		const IDENTIFIER: &'static str = "testextension";
		fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
			Ok(1)
		}
	}

	#[derive(Clone, Eq, Debug, PartialEq, Encode, Decode)]
	struct TestExtension2;
	impl sp_runtime::traits::SignedExtension for TestExtension2 {
		type AccountId = u32;
		type Call = ();
		type AdditionalSigned = u32;
		type Pre = ();
		const IDENTIFIER: &'static str = "testextension2";
		fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
			Ok(1)
		}
	}

	struct TestExtrinsic;

	impl sp_runtime::traits::ExtrinsicMetadata for TestExtrinsic {
		const VERSION: u8 = 1;
		type SignedExtensions = (TestExtension, TestExtension2);
	}

	#[frame_support::pallet(System)]
	mod frame_system {
		use super::*;
		use frame_support::pallet_prelude::*;

		/// Kind of alias for `Config` trait. Deprecated as `Trait` is renamed `Config`.
		pub trait Trait: Config {}
		impl<T: Config> Trait for T {}
		#[pallet::config]
		pub trait Config: 'static {
			type BaseCallFilter;
			const ASSOCIATED_CONST: u64 = 500;
			type Origin: Into<Result<RawOrigin<Self::AccountId>, Self::Origin>>
				+ From<RawOrigin<Self::AccountId>>;
			type AccountId: From<u32> + Encode;
			type BlockNumber: From<u32> + Encode;
			type SomeValue: Get<u32>;
			type PalletInfo: crate::traits::PalletInfo;
			type DbWeight: crate::traits::Get<crate::weights::RuntimeDbWeight>;
			type Call;
		}

		// emulate the actual frame_system module exports
		pub mod pallet_prelude {
			pub type OriginFor<T> = <T as super::Trait>::Origin;
			pub type BlockNumberFor<T> = <T as super::Trait>::BlockNumber;
		}

		#[pallet::module]
		pub struct Module<T>(PhantomData<T>);

		#[pallet::module_interface]
		impl<T: Trait> ModuleInterface<BlockNumberFor<T>> for Module<T> {
		}

		#[pallet::call]
		impl<T: Trait> Call for Module<T> {}

		#[pallet::event]
		pub enum Event {
			SystemEvent,
		}

		#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
		pub enum RawOrigin<AccountId> {
			Root,
			Signed(AccountId),
			None,
		}

		impl<AccountId> From<Option<AccountId>> for RawOrigin<AccountId> {
			fn from(s: Option<AccountId>) -> RawOrigin<AccountId> {
				match s {
					Some(who) => RawOrigin::Signed(who),
					None => RawOrigin::None,
				}
			}
		}

		pub type Origin<T> = RawOrigin<<T as Config>::AccountId>;
	}

	#[frame_support::pallet(EventModule)]
	mod event_module {
		use crate::dispatch::DispatchResult;
		use super::frame_system;

		#[pallet::config]
		pub trait Trait: frame_system::Trait {
			type Balance: Member
				+ codec::Codec
				+ scale_info::TypeInfo // todo: [AJ] only for std, and embed encoded on build?
				+ codec::HasCompact; // todo: [AJ] can we remove this?
		}

		type BalanceOf<T> = <T as Trait>::Balance;

		#[pallet::event]
		#[pallet::metadata(BalanceOf<T> = Balance)]
		pub enum Event<T: Trait> {
			/// Hi, I am a comment.
			TestEvent(BalanceOf<T>),
		}

		#[pallet::module]
		pub struct Module<T>(PhantomData<T>);

		#[pallet::module_interface]
		impl<T: Trait> ModuleInterface<BlockNumberFor<T>> for Module<T> {
		}

		#[pallet::call]
		impl<T: Trait> Call for Module<T> {
			/// Doc comment put in metadata
			#[pallet::weight = 0] // Defines weight for call (function parameters are in scope)
			fn aux_0(origin: OriginFor<T>, #[pallet::compact] balance: T::Balance) -> DispatchResultWithPostInfo {
				let _ = origin;
				let _ = balance;
				unreachable!();
			}
		}

		#[pallet::error]
		pub enum Error<T> {
			/// Some user input error
			UserInputError,
			/// Something bad happened
			/// this could be due to many reasons
			BadThingHappened,
		}
	}

	#[frame_support::pallet(EventModule2)]
	mod event_module2 {
		use super::frame_system;

		#[pallet::config]
		pub trait Trait: frame_system::Trait {
			type Balance: Member + scale_info::TypeInfo;
		}

		#[pallet::module]
		pub struct Module<T>(PhantomData<T>);

		#[pallet::module_interface]
		impl<T: Trait> ModuleInterface<BlockNumberFor<T>> for Module<T> {
		}

		#[pallet::call]
		impl<T: Trait> Call for Module<T> {}

		type BalanceOf<T> = <T as Trait>::Balance;

		#[pallet::event]
		#[pallet::metadata(T::Balance = Balance)]
		pub enum Event<T: Trait> {
			TestEvent(T::Balance),
		}

		#[pallet::storage] #[allow(type_alias_bounds)]
		type TestStorage = StorageValueType<TestStorageP, Option<u32>, ValueQuery>;
	}

	type EventModule = event_module::Module<TestRuntime>;
	type EventModule2 = event_module2::Module<TestRuntime>;

	#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, scale_info::TypeInfo)]
	pub struct TestRuntime;

	impl_outer_event! {
		pub enum TestEvent for TestRuntime {
			frame_system,
			event_module<T>,
			event_module2<T>,
		}
	}

	impl_outer_origin! {
		pub enum Origin for TestRuntime where system = frame_system {}
	}

	impl_outer_dispatch! {
		pub enum Call for TestRuntime where origin: Origin {
			event_module::EventModule,
			event_module2::EventModule2,
		}
	}

	impl event_module::Trait for TestRuntime {
		type Balance = u32;
	}

	impl event_module2::Trait for TestRuntime {
		type Balance = u32;
	}

	crate::parameter_types! {
		pub const SystemValue: u32 = 600;
	}

	impl frame_system::Config for TestRuntime {
		type BaseCallFilter = ();
		type Origin = Origin;
		type AccountId = u32;
		type BlockNumber = u32;
		type SomeValue = SystemValue;
		type PalletInfo = ();
		type DbWeight = ();
		type Call = Call;
	}

	impl_runtime_metadata!(
		for TestRuntime with modules where Extrinsic = TestExtrinsic
			system::Module as System { index 0 } with Event,
			event_module::Module as Module { index 1 } with Event Call,
			event_module2::Module as Module2 { index 2 } with Event Storage Call,
	);

	struct ConstantBlockNumberByteGetter;
	impl DefaultByte for ConstantBlockNumberByteGetter {
		fn default_byte(&self) -> Vec<u8> {
			100u32.encode()
		}
	}

	struct ConstantGetTypeByteGetter;
	impl DefaultByte for ConstantGetTypeByteGetter {
		fn default_byte(&self) -> Vec<u8> {
			SystemValue::get().encode()
		}
	}

	struct ConstantAssociatedConstByteGetter;
	impl DefaultByte for ConstantAssociatedConstByteGetter {
		fn default_byte(&self) -> Vec<u8> {
			<TestRuntime as frame_system::Config>::ASSOCIATED_CONST.encode()
		}
	}

	#[test]
	fn runtime_metadata() {
		let mut registry = Registry::new();
		let expected_metadata = vnext::RuntimeMetadataLastVersion {
			modules: vec![
				vnext::ModuleMetadata {
					name: "System",
					index: 0,
					// storage: None,
					calls: None,
					event: Some(vec![
						vnext::EventMetadata {
							name: "SystemEvent",
							arguments: Vec::new(),
							documentation: Vec::new()
						}
					]),
					// constants: DecodeDifferent::Encode(
					// 	FnEncode(|| &[
					// 		ModuleConstantMetadata {
					// 			name: DecodeDifferent::Encode("BlockNumber"),
					// 			ty: DecodeDifferent::Encode("T::BlockNumber"),
					// 			value: DecodeDifferent::Encode(
					// 				DefaultByteGetter(&ConstantBlockNumberByteGetter)
					// 			),
					// 			documentation: DecodeDifferent::Encode(&[" Hi, I am a comment."]),
					// 		},
					// 		ModuleConstantMetadata {
					// 			name: DecodeDifferent::Encode("GetType"),
					// 			ty: DecodeDifferent::Encode("T::AccountId"),
					// 			value: DecodeDifferent::Encode(
					// 				DefaultByteGetter(&ConstantGetTypeByteGetter)
					// 			),
					// 			documentation: DecodeDifferent::Encode(&[]),
					// 		},
					// 		ModuleConstantMetadata {
					// 			name: DecodeDifferent::Encode("ASSOCIATED_CONST"),
					// 			ty: DecodeDifferent::Encode("u64"),
					// 			value: DecodeDifferent::Encode(
					// 				DefaultByteGetter(&ConstantAssociatedConstByteGetter)
					// 			),
					// 			documentation: DecodeDifferent::Encode(&[]),
					// 		}
					// 	])
					// ),
					// errors: DecodeDifferent::Encode(FnEncode(|| &[])),
				},
				vnext::ModuleMetadata {
					name: "Module",
					index: 1,
					// storage: None,
					calls: Some(vec![
						vnext::FunctionMetadata {
								name: "aux_0",
								arguments: vec! [
									vnext::FunctionArgumentMetadata {
										name: "balance",
										ty: scale_info::meta_type::<<TestRuntime as event_module::Trait>::Balance>(),
										is_compact: true,
									}
								],
								documentation: vec![" Doc comment put in metadata"],
							}]),
					event: Some(vec![
						vnext::EventMetadata {
							name: "TestEvent",
							arguments: vec![
								vnext::TypeSpec::with_name_str::<<TestRuntime as event_module::Trait>::Balance>("Balance")
							],
							documentation: vec![" Hi, I am a comment."],
						}
					]),
					// constants: DecodeDifferent::Encode(FnEncode(|| &[])),
					// errors: DecodeDifferent::Encode(FnEncode(|| &[
					// 	ErrorMetadata {
					// 		name: DecodeDifferent::Encode("UserInputError"),
					// 		documentation: DecodeDifferent::Encode(&[" Some user input error"]),
					// 	},
					// 	ErrorMetadata {
					// 		name: DecodeDifferent::Encode("BadThingHappened"),
					// 		documentation: DecodeDifferent::Encode(&[
					// 			" Something bad happened",
					// 			" this could be due to many reasons",
					// 		]),
					// 	},
					// ])),
				},
				vnext::ModuleMetadata {
					name: "Module2",
					index: 2,
					// storage: Some(DecodeDifferent::Encode(
					// 	FnEncode(|| StorageMetadata {
					// 		prefix: DecodeDifferent::Encode("TestStorage"),
					// 		entries: DecodeDifferent::Encode(
					// 			&[
					// 				StorageEntryMetadata {
					// 					name: DecodeDifferent::Encode("StorageMethod"),
					// 					modifier: StorageEntryModifier::Optional,
					// 					ty: StorageEntryType::Plain(DecodeDifferent::Encode("u32")),
					// 					default: DecodeDifferent::Encode(
					// 						DefaultByteGetter(
					// 							&event_module2::__GetByteStructStorageMethod(
					// 								std::marker::PhantomData::<TestRuntime>
					// 							)
					// 						)
					// 					),
					// 					documentation: DecodeDifferent::Encode(&[]),
					// 				}
					// 			]
					// 		)
					// 	}),
					// )),
					calls: Some(vec![]),
					event: Some(vec![
						vnext::EventMetadata {
							name: "TestEvent",
							arguments: vec![
								vnext::TypeSpec::with_name_str::<<TestRuntime as event_module::Trait>::Balance>("Balance")
							],
							documentation: Vec::new(),
						}
					]),
					// constants: DecodeDifferent::Encode(FnEncode(|| &[])),
					// errors: DecodeDifferent::Encode(FnEncode(|| &[])),
				},
			],
			// extrinsic: ExtrinsicMetadata {
			// 	version: 1,
			// 	signed_extensions: vec![
			// 		DecodeDifferent::Encode("testextension"),
			// 		DecodeDifferent::Encode("testextension2"),
			// 	],
			// }
		};

		let metadata_encoded = TestRuntime::metadata().encode();
		let metadata_decoded = vnext::RuntimeMetadataPrefixed::<scale_info::form::CompactForm>::decode(&mut &metadata_encoded[..]);

		let expected_metadata: vnext::RuntimeMetadataPrefixed<CompactForm> = expected_metadata.into_compact(&mut registry).into();
		let expected_metadata_encoded = expected_metadata.encode();
		let expected_metadata_decoded = vnext::RuntimeMetadataPrefixed::<scale_info::form::CompactForm>::decode(&mut &expected_metadata_encoded[..]);

		pretty_assertions::assert_eq!(expected_metadata_decoded.unwrap(), metadata_decoded.unwrap());
	}
}
