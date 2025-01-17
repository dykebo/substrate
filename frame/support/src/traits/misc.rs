// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
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

//! Smaller traits used in FRAME which don't need their own file.

use crate::dispatch::Parameter;
use codec::{Decode, Encode, EncodeLike, Input, MaxEncodedLen};
use scale_info::{build::Fields, meta_type, Path, Type, TypeInfo, TypeParameter};
use sp_runtime::{traits::Block as BlockT, DispatchError};
use sp_std::prelude::*;

/// Anything that can have a `::len()` method.
pub trait Len {
	/// Return the length of data type.
	fn len(&self) -> usize;
}

impl<T: IntoIterator + Clone> Len for T
where
	<T as IntoIterator>::IntoIter: ExactSizeIterator,
{
	fn len(&self) -> usize {
		self.clone().into_iter().len()
	}
}

/// A trait for querying a single value from a type.
///
/// It is not required that the value is constant.
pub trait Get<T> {
	/// Return the current value.
	fn get() -> T;
}

impl<T: Default> Get<T> for () {
	fn get() -> T {
		T::default()
	}
}

/// Implement Get by returning Default for any type that implements Default.
pub struct GetDefault;
impl<T: Default> Get<T> for GetDefault {
	fn get() -> T {
		T::default()
	}
}

/// Implement `Get<u32>` and `Get<Option<u32>>` using the given const.
pub struct ConstU32<const T: u32>;

impl<const T: u32> Get<u32> for ConstU32<T> {
	fn get() -> u32 {
		T
	}
}

impl<const T: u32> Get<Option<u32>> for ConstU32<T> {
	fn get() -> Option<u32> {
		Some(T)
	}
}

/// A type for which some values make sense to be able to drop without further consideration.
pub trait TryDrop: Sized {
	/// Drop an instance cleanly. Only works if its value represents "no-operation".
	fn try_drop(self) -> Result<(), Self>;
}

/// Return type used when we need to return one of two items, each of the opposite direction or
/// sign, with one (`Same`) being of the same type as the `self` or primary argument of the function
/// that returned it.
pub enum SameOrOther<A, B> {
	/// No item.
	None,
	/// An item of the same type as the `Self` on which the return function was called.
	Same(A),
	/// An item of the opposite type to the `Self` on which the return function was called.
	Other(B),
}

impl<A, B> TryDrop for SameOrOther<A, B> {
	fn try_drop(self) -> Result<(), Self> {
		if let SameOrOther::None = self {
			Ok(())
		} else {
			Err(self)
		}
	}
}

impl<A, B> SameOrOther<A, B> {
	/// Returns `Ok` with the inner value of `Same` if `self` is that, otherwise returns `Err` with
	/// `self`.
	pub fn try_same(self) -> Result<A, Self> {
		match self {
			SameOrOther::Same(a) => Ok(a),
			x => Err(x),
		}
	}

	/// Returns `Ok` with the inner value of `Other` if `self` is that, otherwise returns `Err` with
	/// `self`.
	pub fn try_other(self) -> Result<B, Self> {
		match self {
			SameOrOther::Other(b) => Ok(b),
			x => Err(x),
		}
	}

	/// Returns `Ok` if `self` is `None`, otherwise returns `Err` with `self`.
	pub fn try_none(self) -> Result<(), Self> {
		match self {
			SameOrOther::None => Ok(()),
			x => Err(x),
		}
	}

	pub fn same(self) -> Result<A, B>
	where
		A: Default,
	{
		match self {
			SameOrOther::Same(a) => Ok(a),
			SameOrOther::None => Ok(A::default()),
			SameOrOther::Other(b) => Err(b),
		}
	}

	pub fn other(self) -> Result<B, A>
	where
		B: Default,
	{
		match self {
			SameOrOther::Same(a) => Err(a),
			SameOrOther::None => Ok(B::default()),
			SameOrOther::Other(b) => Ok(b),
		}
	}
}

/// Handler for when a new account has been created.
#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait OnNewAccount<AccountId> {
	/// A new account `who` has been registered.
	fn on_new_account(who: &AccountId);
}

/// The account with the given id was reaped.
#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait OnKilledAccount<AccountId> {
	/// The account with the given id was reaped.
	fn on_killed_account(who: &AccountId);
}

/// A simple, generic one-parameter event notifier/handler.
pub trait HandleLifetime<T> {
	/// An account was created.
	fn created(_t: &T) -> Result<(), DispatchError> {
		Ok(())
	}

	/// An account was killed.
	fn killed(_t: &T) -> Result<(), DispatchError> {
		Ok(())
	}
}

impl<T> HandleLifetime<T> for () {}

pub trait Time {
	type Moment: sp_arithmetic::traits::AtLeast32Bit + Parameter + Default + Copy;

	fn now() -> Self::Moment;
}

/// Trait to deal with unix time.
pub trait UnixTime {
	/// Return duration since `SystemTime::UNIX_EPOCH`.
	fn now() -> core::time::Duration;
}

/// Trait to be used when types are exactly same.
///
/// This allow to convert back and forth from type, a reference and a mutable reference.
pub trait IsType<T>: Into<T> + From<T> {
	/// Cast reference.
	fn from_ref(t: &T) -> &Self;

	/// Cast reference.
	fn into_ref(&self) -> &T;

	/// Cast mutable reference.
	fn from_mut(t: &mut T) -> &mut Self;

	/// Cast mutable reference.
	fn into_mut(&mut self) -> &mut T;
}

impl<T> IsType<T> for T {
	fn from_ref(t: &T) -> &Self {
		t
	}
	fn into_ref(&self) -> &T {
		self
	}
	fn from_mut(t: &mut T) -> &mut Self {
		t
	}
	fn into_mut(&mut self) -> &mut T {
		self
	}
}

/// Something that can be checked to be a of sub type `T`.
///
/// This is useful for enums where each variant encapsulates a different sub type, and
/// you need access to these sub types.
///
/// For example, in FRAME, this trait is implemented for the runtime `Call` enum. Pallets use this
/// to check if a certain call is an instance of the local pallet's `Call` enum.
///
/// # Example
///
/// ```
/// # use frame_support::traits::IsSubType;
///
/// enum Test {
///     String(String),
///     U32(u32),
/// }
///
/// impl IsSubType<String> for Test {
///     fn is_sub_type(&self) -> Option<&String> {
///         match self {
///             Self::String(ref r) => Some(r),
///             _ => None,
///         }
///     }
/// }
///
/// impl IsSubType<u32> for Test {
///     fn is_sub_type(&self) -> Option<&u32> {
///         match self {
///             Self::U32(ref r) => Some(r),
///             _ => None,
///         }
///     }
/// }
///
/// fn main() {
///     let data = Test::String("test".into());
///
///     assert_eq!("test", IsSubType::<String>::is_sub_type(&data).unwrap().as_str());
/// }
/// ```
pub trait IsSubType<T> {
	/// Returns `Some(_)` if `self` is an instance of sub type `T`.
	fn is_sub_type(&self) -> Option<&T>;
}

/// Something that can execute a given block.
///
/// Executing a block means that all extrinsics in a given block will be executed and the resulting
/// header will be checked against the header of the given block.
pub trait ExecuteBlock<Block: BlockT> {
	/// Execute the given `block`.
	///
	/// This will execute all extrinsics in the block and check that the resulting header is
	/// correct.
	///
	/// # Panic
	///
	/// Panics when an extrinsics panics or the resulting header doesn't match the expected header.
	fn execute_block(block: Block);
}

/// Off-chain computation trait.
///
/// Implementing this trait on a module allows you to perform long-running tasks
/// that make (by default) validators generate transactions that feed results
/// of those long-running computations back on chain.
///
/// NOTE: This function runs off-chain, so it can access the block state,
/// but cannot preform any alterations. More specifically alterations are
/// not forbidden, but they are not persisted in any way after the worker
/// has finished.
#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait OffchainWorker<BlockNumber> {
	/// This function is being called after every block import (when fully synced).
	///
	/// Implement this and use any of the `Offchain` `sp_io` set of APIs
	/// to perform off-chain computations, calls and submit transactions
	/// with results to trigger any on-chain changes.
	/// Any state alterations are lost and are not persisted.
	fn offchain_worker(_n: BlockNumber) {}
}

/// Some amount of backing from a group. The precise definition of what it means to "back" something
/// is left flexible.
pub struct Backing {
	/// The number of members of the group that back some motion.
	pub approvals: u32,
	/// The total count of group members.
	pub eligible: u32,
}

/// Retrieve the backing from an object's ref.
pub trait GetBacking {
	/// Returns `Some` `Backing` if `self` represents a fractional/groupwise backing of some
	/// implicit motion. `None` if it does not.
	fn get_backing(&self) -> Option<Backing>;
}

/// A trait to ensure the inherent are before non-inherent in a block.
///
/// This is typically implemented on runtime, through `construct_runtime!`.
pub trait EnsureInherentsAreFirst<Block> {
	/// Ensure the position of inherent is correct, i.e. they are before non-inherents.
	///
	/// On error return the index of the inherent with invalid position (counting from 0).
	fn ensure_inherents_are_first(block: &Block) -> Result<(), u32>;
}

/// An extrinsic on which we can get access to call.
pub trait ExtrinsicCall: sp_runtime::traits::Extrinsic {
	/// Get the call of the extrinsic.
	fn call(&self) -> &Self::Call;
}

#[cfg(feature = "std")]
impl<Call, Extra> ExtrinsicCall for sp_runtime::testing::TestXt<Call, Extra>
where
	Call: codec::Codec + Sync + Send,
{
	fn call(&self) -> &Self::Call {
		&self.call
	}
}

impl<Address, Call, Signature, Extra> ExtrinsicCall
	for sp_runtime::generic::UncheckedExtrinsic<Address, Call, Signature, Extra>
where
	Extra: sp_runtime::traits::SignedExtension,
{
	fn call(&self) -> &Self::Call {
		&self.function
	}
}

/// Something that can estimate the fee of a (frame-based) call.
///
/// Typically, the same pallet that will charge transaction fees will implement this.
pub trait EstimateCallFee<Call, Balance> {
	/// Estimate the fee of this call.
	///
	/// The dispatch info and the length is deduced from the call. The post info can optionally be
	/// provided.
	fn estimate_call_fee(call: &Call, post_info: crate::weights::PostDispatchInfo) -> Balance;
}

// Useful for building mocks.
#[cfg(feature = "std")]
impl<Call, Balance: From<u32>, const T: u32> EstimateCallFee<Call, Balance> for ConstU32<T> {
	fn estimate_call_fee(_: &Call, _: crate::weights::PostDispatchInfo) -> Balance {
		T.into()
	}
}

/// A wrapper for any type `T` which implement encode/decode in a way compatible with `Vec<u8>`.
///
/// The encoding is the encoding of `T` prepended with the compact encoding of its size in bytes.
/// Thus the encoded value can be decoded as a `Vec<u8>`.
#[derive(Debug, Eq, PartialEq, Default, Clone)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct WrapperOpaque<T>(pub T);

impl<T: Encode> EncodeLike for WrapperOpaque<T> {}

impl<T: Encode> Encode for WrapperOpaque<T> {
	fn size_hint(&self) -> usize {
		self.0.size_hint().saturating_add(<codec::Compact<u32>>::max_encoded_len())
	}

	fn encode_to<O: codec::Output + ?Sized>(&self, dest: &mut O) {
		self.0.encode().encode_to(dest);
	}

	fn encode(&self) -> Vec<u8> {
		self.0.encode().encode()
	}

	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.encode().using_encoded(f)
	}
}

impl<T: Decode> Decode for WrapperOpaque<T> {
	fn decode<I: Input>(input: &mut I) -> Result<Self, codec::Error> {
		Ok(Self(T::decode(&mut &<Vec<u8>>::decode(input)?[..])?))
	}

	fn skip<I: Input>(input: &mut I) -> Result<(), codec::Error> {
		<Vec<u8>>::skip(input)
	}
}

impl<T> From<T> for WrapperOpaque<T> {
	fn from(t: T) -> Self {
		Self(t)
	}
}

impl<T: MaxEncodedLen> MaxEncodedLen for WrapperOpaque<T> {
	fn max_encoded_len() -> usize {
		let t_max_len = T::max_encoded_len();

		// See scale encoding https://docs.substrate.io/v3/advanced/scale-codec
		if t_max_len < 64 {
			t_max_len + 1
		} else if t_max_len < 2usize.pow(14) {
			t_max_len + 2
		} else if t_max_len < 2usize.pow(30) {
			t_max_len + 4
		} else {
			<codec::Compact<u32>>::max_encoded_len().saturating_add(T::max_encoded_len())
		}
	}
}

impl<T: TypeInfo + 'static> TypeInfo for WrapperOpaque<T> {
	type Identity = Self;
	fn type_info() -> Type {
		Type::builder()
			.path(Path::new("WrapperOpaque", module_path!()))
			.type_params(vec![TypeParameter::new("T", Some(meta_type::<T>()))])
			.composite(
				Fields::unnamed()
					.field(|f| f.compact::<u32>())
					.field(|f| f.ty::<T>().type_name("T")),
			)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_opaque_wrapper() {
		let encoded = WrapperOpaque(3u32).encode();
		assert_eq!(encoded, [codec::Compact(4u32).encode(), 3u32.to_le_bytes().to_vec()].concat());
		let vec_u8 = <Vec<u8>>::decode(&mut &encoded[..]).unwrap();
		let decoded_from_vec_u8 = u32::decode(&mut &vec_u8[..]).unwrap();
		assert_eq!(decoded_from_vec_u8, 3u32);
		let decoded = <WrapperOpaque<u32>>::decode(&mut &encoded[..]).unwrap();
		assert_eq!(decoded.0, 3u32);

		assert_eq!(<WrapperOpaque<[u8; 63]>>::max_encoded_len(), 63 + 1);
		assert_eq!(
			<WrapperOpaque<[u8; 63]>>::max_encoded_len(),
			WrapperOpaque([0u8; 63]).encode().len()
		);

		assert_eq!(<WrapperOpaque<[u8; 64]>>::max_encoded_len(), 64 + 2);
		assert_eq!(
			<WrapperOpaque<[u8; 64]>>::max_encoded_len(),
			WrapperOpaque([0u8; 64]).encode().len()
		);

		assert_eq!(
			<WrapperOpaque<[u8; 2usize.pow(14) - 1]>>::max_encoded_len(),
			2usize.pow(14) - 1 + 2
		);
		assert_eq!(<WrapperOpaque<[u8; 2usize.pow(14)]>>::max_encoded_len(), 2usize.pow(14) + 4);
	}
}
