{-# LANGUAGE DeriveAnyClass #-}
{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE TypeApplications #-}

module Regression where

import qualified Control.Exception
import qualified Data.Map
import qualified Dhall
import qualified Dhall.Core
import qualified Dhall.Parser
import qualified Test.Tasty
import qualified Test.Tasty.HUnit
import qualified Util

import Dhall.Import (Imported)
import Dhall.Parser (Src)
import Dhall.TypeCheck (TypeError)
import Test.Tasty (TestTree)
import Test.Tasty.HUnit ((@?=))

regressionTests :: TestTree
regressionTests =
    Test.Tasty.testGroup "regression tests"
        [ issue96
        , issue126
        , issue151
        , parsing0
        , typeChecking0
        , typeChecking1
        , typeChecking2
        , unnamedFields
        , trailingSpaceAfterStringLiterals
        ]

data Foo = Foo Integer Bool | Bar Bool Bool Bool | Baz Integer Integer
    deriving (Show, Dhall.Generic, Dhall.Interpret, Dhall.Inject)

unnamedFields :: TestTree
unnamedFields = Test.Tasty.HUnit.testCase "Unnamed Fields" (do
    let ty = Dhall.auto @Foo
    Test.Tasty.HUnit.assertEqual "Good type" (Dhall.expected ty) (Dhall.Core.Union (
            Data.Map.fromList [
                ("Bar",Dhall.Core.Record (Data.Map.fromList [
                    ("_1",Dhall.Core.Bool),("_2",Dhall.Core.Bool),("_3",Dhall.Core.Bool)]))
                , ("Baz",Dhall.Core.Record (Data.Map.fromList [
                    ("_1",Dhall.Core.Integer),("_2",Dhall.Core.Integer)]))
                ,("Foo",Dhall.Core.Record (Data.Map.fromList [
                    ("_1",Dhall.Core.Integer),("_2",Dhall.Core.Bool)]))]))

    let inj = Dhall.inject @Foo
    Test.Tasty.HUnit.assertEqual "Good Inject" (Dhall.declared inj) (Dhall.expected ty)

    let tu_ty = Dhall.auto @(Integer, Bool)
    Test.Tasty.HUnit.assertEqual "Auto Tuple" (Dhall.expected tu_ty) (Dhall.Core.Record (
            Data.Map.fromList [ ("_1",Dhall.Core.Integer),("_2",Dhall.Core.Bool) ]))

    let tu_in = Dhall.inject @(Integer, Bool)
    Test.Tasty.HUnit.assertEqual "Inj. Tuple" (Dhall.declared tu_in) (Dhall.expected tu_ty)

    return () )

issue96 :: TestTree
issue96 = Test.Tasty.HUnit.testCase "Issue #96" (do
    -- Verify that parsing should not fail
    _ <- Util.code "\"bar'baz\""
    return () )

issue126 :: TestTree
issue126 = Test.Tasty.HUnit.testCase "Issue #126" (do
    e <- Util.code
        "''\n\
        \  foo\n\
        \  bar\n\
        \''"
    Util.normalize' e @?= "\"foo\\nbar\\n\"" )

issue151 :: TestTree
issue151 = Test.Tasty.HUnit.testCase "Issue #151" (do
    let shouldNotTypeCheck text = do
            let handler :: Imported (TypeError Src) -> IO Bool
                handler _ = return True

            let typeCheck = do
                    _ <- Util.code text
                    return False
            b <- Control.Exception.handle handler typeCheck
            Test.Tasty.HUnit.assertBool "The expression should not type-check" b
            
    -- These two examples contain the following expression that loops infinitely
    -- if you normalize the expression before type-checking the expression:
    --
    --     (λ(x : A) → x x) (λ(x : A) → x x)
    --
    -- There was a bug in the Dhall type-checker were expressions were not
    -- being type-checked before being added to the context (which is a
    -- violation of the standard type-checking rules for a pure type system).
    -- The reason this is problematic is that several places in the
    -- type-checking logic assume that the context is safe to normalize.
    --
    -- Both of these examples exercise area of the type-checker logic that
    -- assume that the context is normalized.  If you fail to type-check terms
    -- before adding them to the context then this test will "fail" with an
    -- infinite loop.  This test "succeeds" if the examples terminate
    -- immediately with a type-checking failure.
    shouldNotTypeCheck "./tests/regression/issue151a.dhall"
    shouldNotTypeCheck "./tests/regression/issue151b.dhall" )

parsing0 :: TestTree
parsing0 = Test.Tasty.HUnit.testCase "Parsing regression #0" (do
    -- Verify that parsing should not fail
    --
    -- In 267093f8cddf1c2f909f2d997c31fd0a7cb2440a I broke the parser when left
    -- factoring the grammer by failing to handle the source tested by this
    -- regression test.  The root of the problem was that the parser was trying
    -- to parse `List ./Node` as `Field List "/Node"` instead of
    -- `App List (Embed (Path (File Homeless "./Node") Code))`.  The latter is
    -- the correct parse because `/Node` is not a valid field label, but the
    -- mistaken parser was committed to the incorrect parse and never attempted
    -- the correct parse.
    case Dhall.Parser.exprFromText mempty "List ./Node" of
        Left  e -> Control.Exception.throwIO e
        Right _ -> return () )

typeChecking0 :: TestTree
typeChecking0 = Test.Tasty.HUnit.testCase "Type-checking regression #0" (do
    -- There used to be a bug in the type-checking logic where `T` would be
    -- added to the context when inferring the type of `λ(x : T) → x`, but was
    -- missing from the context when inferring the kind of the inferred type
    -- (i.e. the kind of `∀(x : T) → T`).  This led to an unbound variable
    -- error when inferring the kind
    --
    -- This bug was originally reported in issue #10
    _ <- Util.code "let Day : Type = Natural in λ(x : Day) → x"
    return () )

typeChecking1 :: TestTree
typeChecking1 = Test.Tasty.HUnit.testCase "Type-checking regression #1" (do
    -- There used to be a bug in the type-checking logic when inferring the
    -- type of `let`.  Specifically, given an expression of the form:
    --
    --     let x : t = e1 in e2
    --
    -- ... the type-checker would not substitute `x` for `e1` in the inferred
    -- of the `let` expression.  This meant that if the inferred type contained
    -- any references to `x` then these references would escape their scope and
    -- result in unbound variable exceptions
    --
    -- This regression test exercises that code path by creating a `let`
    -- expression where the inferred type before substitution (`∀(x : b) → b` in
    -- this example), contains a reference to the `let`-bound variable (`b`)
    -- that needs to be substituted with `a` in order to produce the final
    -- correct inferred type (`∀(x : a) → a`).  If this test passes then
    -- substitution worked correctly.  If substitution doesn't occur then you
    -- expect an `Unbound variable` error from `b` escaping its scope due to
    -- being returned as part of the inferred type
    _ <- Util.code "λ(a : Type) → let b : Type = a in λ(x : b) → x"
    return () )

typeChecking2 :: TestTree
typeChecking2 = Test.Tasty.HUnit.testCase "Type-checking regression #2" (do
    -- There used to be a bug in the type-checking logic where `let` bound
    -- variables would not correctly shadow variables of the same name when
    -- inferring the type of of the `let` expression
    --
    -- This example exercises this code path with a `let` expression where the
    -- `let`-bound variable (`a`) has the same name as its own type (`a`).  If
    -- shadowing works correctly then the final `a` in the expression should
    -- refer to the `let`-bound variable and not its type.  An invalid reference
    -- to the shadowed type `a` would result in an invalid dependently typed
    -- function
    _ <- Util.code "λ(a : Type) → λ(x : a) → let a : a = x in a"
    return () )

trailingSpaceAfterStringLiterals :: TestTree
trailingSpaceAfterStringLiterals =
    Test.Tasty.HUnit.testCase "Trailing space after string literals" (do
        -- Verify that string literals parse correctly with trailing space
        -- (Yes, I did get this wrong at some point)
        _ <- Util.code "(''ABC'' ++ \"DEF\" )"
        return () )
