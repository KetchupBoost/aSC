# Exemplos de requests
# curl -v -XPOST -H "content-type: application/json" -d '{"apelido" : "xpto", "nome" : "xpto xpto", "nascimento" : "2000-01-01", "stack": null}' "http://localhost:9999/pessoas"
# curl -v -XGET "http://localhost:9999/pessoas/1"
# curl -v -XGET "http://localhost:9999/pessoas?t=xpto"
# curl -v "http://localhost:9999/contagem-pessoas"
GATLING_BIN_DIR=$HOME/utils/gatling/3.10.4/bin
WORKSPACE=$HOME/learn/rust/aSC/stress-test

runGatling() {
    $GATLING_BIN_DIR/gatling.sh -rm local -s RinhaBackendSimulation \
        -rf $WORKSPACE/user-files/results \
        -sf $WORKSPACE/user-files/simulations \
        -rsf $WORKSPACE/user-files/resources 
}

startTest() {
    for i in {1..20}; do
        runGatling && \
        break || sleep 2;
    done
}

startTest

# sh $GATLING_BIN_DIR/gatling.sh -rm local -s StressTest \
#     -rd "DESCRICAO" \
#     -rf $WORKSPACE/user-files/results \
#     -sf $WORKSPACE/user-files/simulations \
#     -rsf $WORKSPACE/user-files/resources \
# sleep 3

# curl -v "http://localhost:9999/contagem-pessoas"
