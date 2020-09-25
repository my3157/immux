## Description

The master repository for Immux code, currently including:
- **ImmuxDB**, the immutable database, and
- **ImmuxCompute**, the immutable computing service.

This repo builds upon an earlier 1.0 version and is under construction.

## ImmuxDB

### Run the servers

```
cargo build
cargo run
```

### HTTP API
```
GET /grouping/id ("get", outside of transactions)
GET /.transactions/tid/grouping/unit_id ("get" within transaction)
GET /grouping/id/.journal ("inspect_one")
GET /.journal ("inspect all")
GET /grouping?filter=xx&lt=xxx&gtxxx
GET /grouping (get all in grouping)

PUT /grouping/id {data} ("set")
PUT /grouping/id?height={height} ("revert one")
PUT /.transactions/tid/grouping/unit_id?height={height} ("revert one" within transaction)
PUT /.transactions/tid/grouping/unit_id {data} ("set" within transaction)
PUT /?height={height} ("revert all")

DELETE /grouping/id ("remove one")
DELETE /.transactions/tid/grouping/unit_id ("remove one" with transaction)
DELETE / ("remove all")

POST /.transactions ("create_transaction")
POST /.transactions/tid?commit ("commit_transaction")
POST /.transactions/tid?abort("abort_transaction")

/* Not implemented yet
GET /grouping/.index (get indices)
POST /grouping/.index?field={field} (create index)
DELETE /grouping/.index/index-number (delete index)
*/
```

### Test

#### Execute storage.tests (both unit storage.tests and end-to-end storage.tests)
```
cargo test
```

#### Gather test coverage

##### 1. Prepare

```bash
rustup toolchain install nightly # add nightly
cargo install grcov # install grcov
```

##### 2. Gather data

```bash
sh coverage.sh
```

##### 3. Check result

Look at `storage.tests/coverage-report/index.html`.

#### Benchmarking

You can run all the benchmarking with a txt output with:

```aidl
sh benches/benches.sh 
```

There are two sets of benchmarks:
1. `artificial` benchmarks uses datasets generated by programs which focuses on regularity of data;
2. `realistic` benchmarks uses realistic datasets and aims to reflect database performance under "real" load.

##### Artificial benchmarks

Currently, we totally have 3 different artificial data set. `indexed_set`, `journal` and `unindex_set`,
 you can bench specific data set by running the command: 

```bash
cargo bench --bench indexed_set -- 100000 100 10 0
```

In the above example, parameters explanation:
 
1. 100000 is `row_count`.
2. 100 is `num_jsons_per_command`.
3. 10 is `report_period`.
4. 0 is `verify_correctness`.

##### Realistic benchmarks

Each benchmark under `benches/realistic` are independent and requires different datasets, which is not pre-loaded in the Git version control because they are large.

Using `berka99` as an example:

###### 1. Download data

```bash
cd benches/realistic/berka99
./init_data.sh
```

###### 2. Execute benchmark

```bash
cargo bench --bench berka99
```

#### Profiling

##### 1. Install flamegraph

```
cargo install flamegraph
```

##### 2. Generate binary executable

```
cargo bench
```

###### 3. Run flamegraph

```bash
# Use sudo if prompted
flamegraph target/release/realistic-SOME_HASH
```

Inspect the "flamegraph.svg" in a web browser (it's interactive).

The exact executable name needs to be checked manually at the moment.
