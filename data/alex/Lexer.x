{
    {-# OPTIONS_GHC -fno-warn-unused-imports -fno-warn-name-shadowing -fno-warn-incomplete-uni-patterns #-}
    {-# LANGUAGE DeriveGeneric #-}
    {-# LANGUAGE DeriveAnyClass #-}
    {-# LANGUAGE StandaloneDeriving #-}
    {-# LANGUAGE OverloadedStrings #-}
    {-# LANGUAGE FlexibleContexts #-}

    module Language.Dhall.Lexer
        ( lexDhall
        , lexFile
        , step
        , Alex (..)
        , Token (..)
        , AlexPosn (..)
        ) where

import Control.DeepSeq (NFData)
import Control.Monad
import Data.Bool (bool)
import Data.Char (chr)
import qualified Data.ByteString.Lazy as BSL
import qualified Data.ByteString.Lazy.Char8 as ASCII
import GHC.Generics (Generic)
import GHC.Natural (Natural)

}

%wrapper "monad-bytestring"

$digit = 0-9

@integer = (\- | "") $digit+

-- TODO // for lambdas?
$special = [\{\}\,\=\:\[\]λ→\<\>\|\(\)\.->∀]

$lowercase = [a-z]
$uppercase = [A-Z]
$letter = [$lowercase $uppercase]

@type = $uppercase $letter*
@identifier = ($lowercase | _ | @type \/) $letter*

@loc = "http://" | "https://" | "./" | "../" | "/"

$url_contents = [\:\/\-\.\_ $letter]

@url = @loc $url_contents+

$string_char = $printable # [\"\\\$]

$esc_char = [\"\$\\]

-- Deficiency: what if $ is followed by "?
@string_in = (\\ $esc_char | $string_char | \$ [^\{\"])*

tokens :-

    <0,splice> $white+           ;

    <0,splice> "--".*            ;
    "{-"                         { nested_comment }

    -- keywords
    <0,splice> let               { tok (\p _ -> alex $ Keyword p KwLet) }
    <0,splice> in                { tok (\p _ -> alex $ Keyword p KwIn) }
    <0,splice> forall            { tok (\p _ -> alex $ Keyword p KwForall) }
    <0,splice> constructors      { tok (\p _ -> alex $ Keyword p KwConstructors) }
    <0,splice> merge             { tok (\p _ -> alex $ Keyword p KwMerge) }

    -- builtin specials
    <0,splice> "//"              { tok (\p _ -> alex $ Operator p CombineTok) }
    <0,splice> "⫽"               { tok (\p _ -> alex $ Operator p CombineTok) }
    <0,splice> "/"\\             { tok (\p _ -> alex $ Operator p PreferTok) }
    <0,splice> "∧"               { tok (\p _ -> alex $ Operator p PreferTok) }

    -- Path literals
    <0,splice> @url              { tok (\p s -> alex $ Loc p s) }

    -- Various special characters
    <0> $special                 { tok (\p s -> alex $ Special p s) }

    -- Numeric literals
    <0,splice> \+ @integer       { tok (\p s -> NatLit p <$> (readNatural s)) }
    <0,splice> @integer          { tok (\p s -> IntLit p <$> (readInteger s)) }

    -- Boolean literals
    <0,splice> True              { tok (\p _ -> alex $ BoolLit p True) }
    <0,splice> False             { tok (\p _ -> alex $ BoolLit p False) }

    -- Identifiers
    <0,splice> @identifier       { tok (\p s -> alex $ Identifier p s) }
    <0,splice> @type             { tok (\p s -> alex $ TypeId p s) }

    -- Strings & string splices
    <0,splice> \"                { begin string }
    <string> @string_in \$ / \"  { tok (\p s -> alex $ StringChunk p s) }
    <string> @string_in          { tok (\p s -> alex $ StringChunk p s) }
    <string> \$\{                { tok (\p _ -> alex $ BeginSplice p) `andBegin` splice }
    <splice> \}                  { tok (\p _ -> alex $ EndSplice p) `andBegin` string }
    <splice> $special # \}       { tok (\p s -> alex $ Special p s) }
    <string> \"                  { begin 0 }

{

-- Taken from example
-- [here](https://github.com/simonmar/alex/blob/master/examples/haskell.x) by
-- Simon Marlow
nested_comment :: AlexInput -> Int64 -> Alex Token
nested_comment _ _ = go 1 =<< alexGetInput

    where go :: Int -> AlexInput -> Alex Token
          go 0 input = alexSetInput input >> alexMonadScan
          go n input = do
            case alexGetByte input of
                Nothing -> err input
                Just (c, input) -> do
                    case chr (fromIntegral c) of
                        '-' -> do
                            case alexGetByte input of
                                Nothing -> err input
                                Just (125,input) -> go (n-1) input
                                Just (_,input) -> go n input
                        '{' -> do
                            case alexGetByte input of
                                Nothing -> err input
                                Just (c,input) -> go (bool id (+1) (c==45) $ n) input
                        _ -> go n input

          err (pos,_,_,_) =
            let (AlexPn _ line col) = pos
            in alexError ("Error in nested comment at line " ++ show line ++ ", column " ++ show col)

readNatural :: BSL.ByteString -> Alex Natural
readNatural = go <=< readInteger
    where go x | x < 0 = alexError "Not a valid natural"
          go x = pure (fromIntegral x)

readInteger :: BSL.ByteString -> Alex Integer
readInteger str =
    case ASCII.readInteger str of
        Just (i, "") -> pure i
        _ -> alexError "Not a valid integer"

alex :: a -> Alex a
alex = pure

tok f (p,_,s,_) len = f p (BSL.take len s)

deriving instance Generic AlexPosn
deriving instance NFData AlexPosn

data Operator = CombineTok
              | PreferTok
              deriving (Eq, Show, Generic, NFData)

data Keyword = KwLet
             | KwIn
             | KwConstructors
             | KwMerge
             | KwForall
             deriving (Eq, Show, Generic, NFData)

data PreToken = IntLit Integer
           | NatLit Natural
           | BoolLit Bool
           | Embedded BSL.ByteString
           | StringLit BSL.ByteString
           | TypeId BSL.ByteString
           | Identifier BSL.ByteString
           | Keyword Keyword
           | Special BSL.ByteString
           | Operator Operator
           | BeginSplice
           | EndSplice
           | StringChunk BSL.ByteString
           | End

data Ann a = Ann { loc :: AlexPosn
                 , inner :: a
                 }

data Token = IntLit AlexPosn Integer
           | NatLit AlexPosn Natural
           | BoolLit AlexPosn Bool
           | Loc AlexPosn BSL.ByteString
           | StringLit AlexPosn BSL.ByteString
           | TypeId AlexPosn BSL.ByteString
           | Identifier AlexPosn BSL.ByteString
           | Keyword AlexPosn Keyword
           | Special AlexPosn BSL.ByteString
           | Operator AlexPosn Operator
           | BeginSplice AlexPosn
           | EndSplice AlexPosn
           | StringChunk AlexPosn BSL.ByteString
           | End
           deriving (Eq, Show, Generic, NFData)

alexEOF :: Alex Token
alexEOF = pure End

lexFile :: FilePath -> IO (Either String [Token])
lexFile = fmap lexDhall . BSL.readFile

lexDhall :: BSL.ByteString -> Either String [Token]
lexDhall str = runAlex str loop

{-# INLINABLE step #-}
step :: Alex Token
step = alexMonadScan

loop :: Alex [Token]
loop = do
    tok' <- step
    if tok' == End then pure mempty
        else (tok' :) <$> loop

}
