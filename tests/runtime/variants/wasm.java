package wit_exports;

import wit_imports.Imports;

public class ExportsImpl {
    public static void testImports() {
        expect(Imports.roundtripOption(1.0F) == (byte) 1);
        expect(Imports.roundtripOption(null) == null);
        expect(Imports.roundtripOption(2.0F) == (byte) 2);

        {
            Imports.Result<Double, Byte> result = Imports.roundtripResult(Imports.Result.ok(2));
            expect(result.tag == Imports.Result.OK && result.getOk() == 2.0D);
        }

        {
            Imports.Result<Double, Byte> result = Imports.roundtripResult(Imports.Result.ok(4));
            expect(result.tag == Imports.Result.OK && result.getOk() == 4.0D);
        }

        {
            Imports.Result<Double, Byte> result = Imports.roundtripResult(Imports.Result.err(5.3F));
            expect(result.tag == Imports.Result.ERR && result.getErr() == (byte) 5);
        }


        expect(Imports.roundtripEnum(Imports.E1.A) == Imports.E1.A);
        expect(Imports.roundtripEnum(Imports.E1.B) == Imports.E1.B);

        expect(Imports.invertBool(true) == false);
        expect(Imports.invertBool(false) == true);

        {
            Imports.Tuple6<Imports.C1, Imports.C2, Imports.C3, Imports.C4, Imports.C5, Imports.C6> result
                = Imports.variantCasts(new Imports.Tuple6<>(Imports.C1.a(1),
                                                            Imports.C2.a(2),
                                                            Imports.C3.a(3),
                                                            Imports.C4.a(4L),
                                                            Imports.C5.a(5L),
                                                            Imports.C6.a(6.0F)));

            expect(result.f0.tag == Imports.C1.A && result.f0.getA() == 1);
            expect(result.f1.tag == Imports.C2.A && result.f1.getA() == 2);
            expect(result.f2.tag == Imports.C3.A && result.f2.getA() == 3);
            expect(result.f3.tag == Imports.C4.A && result.f3.getA() == 4L);
            expect(result.f4.tag == Imports.C5.A && result.f4.getA() == 5L);
            expect(result.f5.tag == Imports.C6.A && result.f5.getA() == 6.0F);
        }

        {
            Imports.Tuple6<Imports.C1, Imports.C2, Imports.C3, Imports.C4, Imports.C5, Imports.C6> result
                = Imports.variantCasts(new Imports.Tuple6<>(Imports.C1.b(1L),
                                                            Imports.C2.b(2.0F),
                                                            Imports.C3.b(3.0D),
                                                            Imports.C4.b(4.0F),
                                                            Imports.C5.b(5.0D),
                                                            Imports.C6.b(6.0D)));

            expect(result.f0.tag == Imports.C1.B && result.f0.getB() == 1L);
            expect(result.f1.tag == Imports.C2.B && result.f1.getB() == 2.0F);
            expect(result.f2.tag == Imports.C3.B && result.f2.getB() == 3.0D);
            expect(result.f3.tag == Imports.C4.B && result.f3.getB() == 4.0F);
            expect(result.f4.tag == Imports.C5.B && result.f4.getB() == 5.0D);
            expect(result.f5.tag == Imports.C6.B && result.f5.getB() == 6.0D);
        }

        {
            Imports.Tuple4<Imports.Z1, Imports.Z2, Imports.Z3, Imports.Z4> result
                = Imports.variantZeros(new Imports.Tuple4<>(Imports.Z1.a(1),
                                                            Imports.Z2.a(2L),
                                                            Imports.Z3.a(3.0F),
                                                            Imports.Z4.a(4.0D)));

            expect(result.f0.tag == Imports.Z1.A && result.f0.getA() == 1);
            expect(result.f1.tag == Imports.Z2.A && result.f1.getA() == 2L);
            expect(result.f2.tag == Imports.Z3.A && result.f2.getA() == 3.0F);
            expect(result.f3.tag == Imports.Z4.A && result.f3.getA() == 4.0D);
        }

        {
            Imports.Tuple4<Imports.Z1, Imports.Z2, Imports.Z3, Imports.Z4> result
                = Imports.variantZeros(new Imports.Tuple4<>(Imports.Z1.b(),
                                                            Imports.Z2.b(),
                                                            Imports.Z3.b(),
                                                            Imports.Z4.b()));

            expect(result.f0.tag == Imports.Z1.B);
            expect(result.f1.tag == Imports.Z2.B);
            expect(result.f2.tag == Imports.Z3.B);
            expect(result.f3.tag == Imports.Z4.B);
        }

        Imports.variantTypedefs(null, false, Imports.Result.err(Imports.Tuple0.INSTANCE));

        {
            Imports.Tuple3<Boolean, Imports.Result<Imports.Tuple0, Imports.Tuple0>, Imports.MyErrno> result
                = Imports.variantEnums(true, Imports.Result.ok(Imports.Tuple0.INSTANCE), Imports.MyErrno.SUCCESS);

            expect(result.f0 == false);
            expect(result.f1.tag == Imports.Result.ERR);
            expect(result.f2 == Imports.MyErrno.A);
        }
    }

    public static Byte roundtripOption(Float a) {
        return a == null ? null : (byte) (float) a;
    }

    public static Exports.Result<Double, Byte> roundtripResult(Exports.Result<Integer, Float> a) {
        switch (a.tag) {
        case Exports.Result.OK: return Exports.Result.ok((double) a.getOk());
        case Exports.Result.ERR: return Exports.Result.err((byte) (float) a.getErr());
        default: throw new AssertionError();
        }
    }

    public static Exports.E1 roundtripEnum(Exports.E1 a) {
        return a;
    }

    public static boolean invertBool(boolean a) {
        return !a;
    }

    public static Exports.Tuple6<Exports.C1, Exports.C2, Exports.C3, Exports.C4, Exports.C5, Exports.C6>
        variantCasts
        (Exports.Tuple6<Exports.C1, Exports.C2, Exports.C3, Exports.C4, Exports.C5, Exports.C6> a)
    {
        return a;
    }

    public static Exports.Tuple4<Exports.Z1, Exports.Z2, Exports.Z3, Exports.Z4> variantZeros
        (Exports.Tuple4<Exports.Z1, Exports.Z2, Exports.Z3, Exports.Z4> a)
    {
        return a;
    }

    public static void variantTypedefs(Integer a, boolean b, Exports.Result<Integer, Exports.Tuple0> c) { }

    private static void expect(boolean v) {
        if (!v) {
            throw new AssertionError();
        }
    }
}
